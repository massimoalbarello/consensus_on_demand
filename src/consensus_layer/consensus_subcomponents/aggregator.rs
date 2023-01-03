//! The share aggregator is responsible for the aggregation of different types
//! of shares into full objects. That is, it constructs Random Beacon objects
//! from random beacon shares, Notarizations from notarization shares and
//! Finalizations from finalization shares.

use std::collections::{BTreeMap, BTreeSet};
use serde::{Deserialize, Serialize};

use crate::SubnetParams;
use crate::consensus_layer::height_index::Height;
use crate::consensus_layer::{
    pool_reader::PoolReader,
    artifacts::ConsensusMessage,
};
use crate::crypto::{Signed, CryptoHashOf};

use super::block_maker::Block;
use super::notary::{NotarizationShareContent, NotarizationShareContentICC, NotarizationShareContentCOD};

// NotarizationContent holds the values that are signed in a notarization
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct NotarizationContent {
    pub height: Height,
    pub block: CryptoHashOf<Block>,
}

impl NotarizationContent {
    pub fn new(block_height: Height, block_hash: CryptoHashOf<Block>) -> Self {
        Self {
            height: block_height,
            block: block_hash,
        }
    }
}

pub type Notarization = Signed<NotarizationContent, u8>;

/// FinalizationContent holds the values that are signed in a finalization
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct FinalizationContent {
    pub height: Height,
    pub block: CryptoHashOf<Block>,
}

impl FinalizationContent {
    pub fn new(height: Height, block: CryptoHashOf<Block>) -> Self {
        FinalizationContent {
            height,
            block,
        }
    }
}

/// A finalization is a multi-signature on a FinalizationContent. A finalization
/// proves that the block identified by the block hash in the finalization
/// content (and the block chain it implies) is agreed upon.
pub type Finalization = Signed<FinalizationContent, u8>;

pub struct ShareAggregator {
    node_id: u8,
    subnet_params: SubnetParams,
}

impl ShareAggregator {
    pub fn new(node_id: u8, subnet_params: SubnetParams) -> Self {
        Self {
            node_id,
            subnet_params,
        }
    }

    /// Attempt to construct artifacts from artifact shares in the artifact
    /// pool
    pub fn on_state_change(&self, pool: &PoolReader<'_>) -> Vec<ConsensusMessage> {
        // println!("\n########## Aggregator ##########");
        let mut messages = Vec::new();
        messages.append(&mut self.aggregate_notarization_shares(pool));
        messages.append(&mut self.aggregate_finalization_shares(pool));
        messages
    }

    /// Attempt to construct `Notarization`s at `notarized_height + 1`
    fn aggregate_notarization_shares(&self, pool: &PoolReader<'_>) -> Vec<ConsensusMessage> {
        let height = pool.get_notarized_height() + 1;
        let notarization_shares = pool.get_notarization_shares(height);
        let grouped_shares_separated_from_acks = aggregate(notarization_shares);    // in case CoD is used, shares and acks for the same proposal are in two separate entries
        // println!("Grouped shares separated from acks {:?}", grouped_shares_separated_from_acks);
        let grouped_shares = group_shares_and_acks(grouped_shares_separated_from_acks);
        // println!("Grouped shares: {:?}", grouped_shares);
        grouped_shares.into_iter().filter_map(|(notary_content, shares)| {
            if shares.len() >= (self.subnet_params.total_nodes_number - self.subnet_params.byzantine_nodes_number) as usize {
                println!("\nNotarization of block with hash: {} at height {} by committee: {:?}", notary_content.block.get_ref(), notary_content.height, shares);
                Some(notary_content)
            }
            else {
                None
            }.map(|notary_content| {
                ConsensusMessage::Notarization(
                    Notarization {
                        content: NotarizationContent {
                            height: notary_content.height,
                            block: notary_content.block,
                        },
                        signature: 0,   // committee signature
                    }
                )
            })
        }).collect()
    }

    /// Attempt to construct `Finalization`s
    fn aggregate_finalization_shares(&self, pool: &PoolReader<'_>) -> Vec<ConsensusMessage> {
        let finalization_shares = pool.get_finalization_shares(
            pool.get_finalized_height() + 1,
            pool.get_notarized_height(),
        );
        let grouped_shares = aggregate(finalization_shares);
        grouped_shares.into_iter().filter_map(|(finalization_content, shares)| {
            if shares.len() >= (self.subnet_params.total_nodes_number - self.subnet_params.byzantine_nodes_number) as usize {
                println!("Finalization of block with hash: {} at height {} by committee: {:?}", finalization_content.block.get_ref(), finalization_content.height, shares);
                Some(finalization_content)
            }
            else {
                None
            }.map(|finalization_content| {
                ConsensusMessage::Finalization(
                    Finalization {
                        content: FinalizationContent {
                            height: finalization_content.height,
                            block: finalization_content.block,
                        },
                        signature: 10,   // committee signature
                    }
                )
            })
        }).collect()
    }

}

pub fn aggregate<T: Ord>(shares: Box<dyn Iterator<Item = Signed<T, u8>>>) -> BTreeMap<T, BTreeSet<u8>>{
    shares.fold(BTreeMap::<T, BTreeSet<u8>>::new(), |mut grouped_shares, share| {
        match grouped_shares.get_mut(&share.content) {
            Some(existing) => {
                existing.insert(share.signature);
            }
            None => {
                let mut new_set = BTreeSet::<u8>::new();
                new_set.insert(share.signature);
                grouped_shares.insert(share.content, new_set);
            }
        };
        grouped_shares
    })
}

fn group_shares_and_acks(grouped_shares_separated_from_acks: BTreeMap<NotarizationShareContent, BTreeSet<u8>>) -> BTreeMap<NotarizationContent, BTreeSet<u8>> {
    grouped_shares_separated_from_acks.iter()
    .filter_map(|(notarization_share_content, committee)| match notarization_share_content {
            // committee contains the "signatures" which can be either shares or acks, depending on the value of the "is_ack" property of NotarizationShareContentCOD
            NotarizationShareContent::COD(notary_content) => {
                // we need to aggregate shares and acks for the same block proposal
                // if there are only acks for a proposal, we might still need to aggregate them into a notarization as
                // the acknowledger might not be able to create an acknowledgement even if it received n-p acks
                // this happens due to rule 2 of CoD which requires the parent of a block to be finalized in order for the block to be acknowledged (FP-finalized)
                match notary_content.is_ack {
                    // group the shares and acks of a proposal for which there is at least one notarization share
                    false => {
                        // look for the entry with the acks for the same proposal
                        let notarization_content_with_ack = NotarizationShareContent::COD(NotarizationShareContentCOD::new(notary_content.height, notary_content.block.clone(), notary_content.block_parent_hash.clone(), Some(true)));
                        match grouped_shares_separated_from_acks.get(&notarization_content_with_ack) {
                            // if there are acks for the same proposal, append them to the shares and insert the set as the value of the aggregator::NotarizationContent
                            Some(acks) => {
                                println!("Merging shares from: {:?} and acks from: {:?} for the same proposal", committee, acks);
                                let mut shares_and_acks = committee.clone();   // notarization shares for proposal
                                let mut acks_mut = acks.clone();    // acks fro the same proposal
                                shares_and_acks.append(&mut acks_mut);
                                Some((NotarizationContent::new(notary_content.height, notary_content.block.clone()), shares_and_acks))
                            },
                            // if there are no acks for the same proposal, insert the shares as the value of the aggregator::NotarizationContent
                            None => Some((NotarizationContent::new(notary_content.height, notary_content.block.clone()), committee.clone())),
                        }
                    },
                    // add notarization content for which there is only acks as these might still be aggregated into a notarization
                    // this happens when the acknowledger cannot acknowledge the block because its parent is not finalized
                    true => {
                        // look for the entry with the shares for the same proposal
                        let notarization_content_without_ack = NotarizationShareContent::COD(NotarizationShareContentCOD::new(notary_content.height, notary_content.block.clone(), notary_content.block_parent_hash.clone(), Some(false)));
                        match grouped_shares_separated_from_acks.get(&notarization_content_without_ack) {
                            // if there are shares for the same proposal, ignore this entry as they are grouped by the arm corresponding to the "false" pattern
                            Some(_) => None,
                            // if there are no shares for the same proposal, insert the acks as the value of the aggregator::NotarizationContent 
                            None => Some((NotarizationContent::new(notary_content.height, notary_content.block.clone()), committee.clone())),
                        }
                    },
                }
            },
            NotarizationShareContent::ICC(notary_content) => {
                // if only ICC is used, as there are no acks, there is no need to group them with the shares
                // shares for the same proposal are already aggregated by the "aggregate" function
                Some((NotarizationContent::new(notary_content.height, notary_content.block.clone()), committee.clone()))
            }
        })
        .collect()  // shares and acks for the same proposal are in the same entry
}



#[cfg(test)]
mod tests {
    use crate::crypto::Id;
    use super::*;

    // test whether shares for the same proposal without any acks are included in the same entry of "grouped_shares"
    #[test]
    fn groups_shares_without_acks() {
        let mut grouped_shares_separated_from_acks = BTreeMap::new();

        // proposal "28d7bd1c45d7e5652aa5e9ed84cfbc666f3e376990cd95fff60e83c0194f3a6c" received shares from replicas 1 and 3
        let mut shares_set = BTreeSet::new();
        shares_set.insert(1 as u8);
        shares_set.insert(3 as u8);
        grouped_shares_separated_from_acks.insert(
            NotarizationShareContent::COD(NotarizationShareContentCOD {
                height: 9,
                block: Id::new(String::from("28d7bd1c45d7e5652aa5e9ed84cfbc666f3e376990cd95fff60e83c0194f3a6c")),
                block_parent_hash: String::from(""),
                is_ack: false 
            }),
            shares_set
        );

        // proposal "6b6dcab6e7b86ee50066b978080a826894aed3162e1fe7046ffed115837bc906" received shares from replica 2
        let mut shares_set = BTreeSet::new();
        shares_set.insert(2 as u8);
        grouped_shares_separated_from_acks.insert(
            NotarizationShareContent::COD(NotarizationShareContentCOD {
                height: 9,
                block: Id::new(String::from("6b6dcab6e7b86ee50066b978080a826894aed3162e1fe7046ffed115837bc906")),
                block_parent_hash: String::from(""),
                is_ack: false
            }),
            shares_set
        );


        let grouped_shares = group_shares_and_acks(grouped_shares_separated_from_acks);

        let mut correct_grouped_shares = BTreeMap::new();

        // shares from replicas 1 and 3 for proposal "28d7bd1c45d7e5652aa5e9ed84cfbc666f3e376990cd95fff60e83c0194f3a6c" must have been included in the same entry
        let mut correct_set = BTreeSet::new();
        correct_set.insert(1 as u8);
        correct_set.insert(3 as u8);
        correct_grouped_shares.insert(
            NotarizationContent {
                height: 9,
                block: Id::new(String::from("28d7bd1c45d7e5652aa5e9ed84cfbc666f3e376990cd95fff60e83c0194f3a6c")),
            },
            correct_set
        );

        // share from replica 2 for proposal "6b6dcab6e7b86ee50066b978080a826894aed3162e1fe7046ffed115837bc906" must have been included in a separate entry
        let mut correct_set = BTreeSet::new();
        correct_set.insert(2 as u8);
        correct_grouped_shares.insert(
            NotarizationContent {
                height: 9,
                block: Id::new(String::from("6b6dcab6e7b86ee50066b978080a826894aed3162e1fe7046ffed115837bc906")),
            },
            correct_set
        );

        assert_eq!(grouped_shares, correct_grouped_shares);
    }

    // test whether acks for a proposal which hasn't received any shares are included in "grouped_shares"
    #[test]
    fn groups_acks_without_shares() {
        let mut grouped_shares_separated_from_acks = BTreeMap::new();

        // proposal "28d7bd1c45d7e5652aa5e9ed84cfbc666f3e376990cd95fff60e83c0194f3a6c" received an ack from replica 2
        let mut acks_set = BTreeSet::new();
        acks_set.insert(2 as u8);
        grouped_shares_separated_from_acks.insert(
            NotarizationShareContent::COD(NotarizationShareContentCOD {
                height: 9,
                block: Id::new(String::from("28d7bd1c45d7e5652aa5e9ed84cfbc666f3e376990cd95fff60e83c0194f3a6c")),
                block_parent_hash: String::from(""),
                is_ack: true 
            }),
            acks_set
        );

        // proposal "6b6dcab6e7b86ee50066b978080a826894aed3162e1fe7046ffed115837bc906" received a acks from replicas 1, 2 and 4
        let mut acks_set = BTreeSet::new();
        acks_set.insert(1 as u8);
        acks_set.insert(2 as u8);
        acks_set.insert(4 as u8);
        grouped_shares_separated_from_acks.insert(
            NotarizationShareContent::COD(NotarizationShareContentCOD {
                height: 9,
                block: Id::new(String::from("6b6dcab6e7b86ee50066b978080a826894aed3162e1fe7046ffed115837bc906")),
                block_parent_hash: String::from(""),
                is_ack: true 
            }),
            acks_set
        );

        let grouped_shares = group_shares_and_acks(grouped_shares_separated_from_acks);

        let mut correct_grouped_shares = BTreeMap::new();

        // ack from replica 2 for proposal "28d7bd1c45d7e5652aa5e9ed84cfbc666f3e376990cd95fff60e83c0194f3a6c" must have been included
        let mut correct_set = BTreeSet::new();
        correct_set.insert(2 as u8);
        correct_grouped_shares.insert(
            NotarizationContent {
                height: 9,
                block: Id::new(String::from("28d7bd1c45d7e5652aa5e9ed84cfbc666f3e376990cd95fff60e83c0194f3a6c")),
            },
            correct_set
        );

        // acks from replicas 1, 2 and 4 for proposal "6b6dcab6e7b86ee50066b978080a826894aed3162e1fe7046ffed115837bc906" must have been included in a separate entry
        let mut correct_set = BTreeSet::new();
        correct_set.insert(1 as u8);
        correct_set.insert(2 as u8);
        correct_set.insert(4 as u8);
        correct_grouped_shares.insert(
            NotarizationContent {
                height: 9,
                block: Id::new(String::from("6b6dcab6e7b86ee50066b978080a826894aed3162e1fe7046ffed115837bc906")),
            },
            correct_set
        );

        assert_eq!(grouped_shares, correct_grouped_shares);
    }

    // test whether acks for proposals which also received shares are included in the same entry of the respective shares
    // also create entry for proposal which received only acks
    #[test]
    fn groups_shares_and_acks_for_same_proposal() {
        let mut grouped_shares_separated_from_acks = BTreeMap::new();

        // proposal "28d7bd1c45d7e5652aa5e9ed84cfbc666f3e376990cd95fff60e83c0194f3a6c" received an ack from replica 2
        let mut acks_set = BTreeSet::new();
        acks_set.insert(2 as u8);
        grouped_shares_separated_from_acks.insert(
            NotarizationShareContent::COD(NotarizationShareContentCOD {
                height: 9,
                block: Id::new(String::from("28d7bd1c45d7e5652aa5e9ed84cfbc666f3e376990cd95fff60e83c0194f3a6c")),
                block_parent_hash: String::from(""),
                is_ack: true 
            }),
            acks_set
        );

        // proposal "28d7bd1c45d7e5652aa5e9ed84cfbc666f3e376990cd95fff60e83c0194f3a6c" received shares from replicas 1 and 3
        let mut shares_set = BTreeSet::new();
        shares_set.insert(1 as u8);
        shares_set.insert(3 as u8);
        grouped_shares_separated_from_acks.insert(
            NotarizationShareContent::COD(NotarizationShareContentCOD {
                height: 9,
                block: Id::new(String::from("28d7bd1c45d7e5652aa5e9ed84cfbc666f3e376990cd95fff60e83c0194f3a6c")),
                block_parent_hash: String::from(""),
                is_ack: false 
            }),
            shares_set
        );

        // proposal "6b6dcab6e7b86ee50066b978080a826894aed3162e1fe7046ffed115837bc906" received a share from replica 2
        let mut shares_set = BTreeSet::new();
        shares_set.insert(2 as u8);
        grouped_shares_separated_from_acks.insert(
            NotarizationShareContent::COD(NotarizationShareContentCOD {
                height: 9,
                block: Id::new(String::from("6b6dcab6e7b86ee50066b978080a826894aed3162e1fe7046ffed115837bc906")),
                block_parent_hash: String::from(""),
                is_ack: false
            }),
            shares_set
        );

        // proposal "fb4f2dafc775ca19792729f7adb3a9bbe9d24725cdc95fa5cff873da32352720" received an ack from replica 4
        let mut acks_set = BTreeSet::new();
        acks_set.insert(4 as u8);
        grouped_shares_separated_from_acks.insert(
            NotarizationShareContent::COD(NotarizationShareContentCOD {
                height: 9,
                block: Id::new(String::from("fb4f2dafc775ca19792729f7adb3a9bbe9d24725cdc95fa5cff873da32352720")),
                block_parent_hash: String::from(""),
                is_ack: true
            }),
            acks_set
        );

        let grouped_shares = group_shares_and_acks(grouped_shares_separated_from_acks);

        let mut correct_grouped_shares = BTreeMap::new();

        // ack from replica 2 for proposal "28d7bd1c45d7e5652aa5e9ed84cfbc666f3e376990cd95fff60e83c0194f3a6c" must have been included in the same entry as the shares for the same proposal from replicas 1 and 3
        let mut correct_set = BTreeSet::new();
        correct_set.insert(1 as u8);
        correct_set.insert(2 as u8);
        correct_set.insert(3 as u8);
        correct_grouped_shares.insert(
            NotarizationContent {
                height: 9,
                block: Id::new(String::from("28d7bd1c45d7e5652aa5e9ed84cfbc666f3e376990cd95fff60e83c0194f3a6c")),
            },
            correct_set
        );

        // share from replica 2 for proposal "6b6dcab6e7b86ee50066b978080a826894aed3162e1fe7046ffed115837bc906" must have been included in a separate entry
        let mut correct_set = BTreeSet::new();
        correct_set.insert(2 as u8);
        correct_grouped_shares.insert(
            NotarizationContent {
                height: 9,
                block: Id::new(String::from("6b6dcab6e7b86ee50066b978080a826894aed3162e1fe7046ffed115837bc906")),
            },
            correct_set
        );

        // ack from replica 4 for proposal "fb4f2dafc775ca19792729f7adb3a9bbe9d24725cdc95fa5cff873da32352720" must have been included in a separate entry
        let mut correct_set = BTreeSet::new();
        correct_set.insert(4 as u8);
        correct_grouped_shares.insert(
            NotarizationContent {
                height: 9,
                block: Id::new(String::from("fb4f2dafc775ca19792729f7adb3a9bbe9d24725cdc95fa5cff873da32352720")),
            },
            correct_set
        );

        assert_eq!(grouped_shares, correct_grouped_shares);
    }
}