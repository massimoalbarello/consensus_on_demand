//! The share aggregator is responsible for the aggregation of different types
//! of shares into full objects. That is, it constructs Random Beacon objects
//! from random beacon shares, Notarizations from notarization shares and
//! Finalizations from finalization shares.

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

use crate::consensus_layer::consensus_subcomponents::goodifier::{
    block_is_good, get_block_by_hash_and_height,
};
use crate::consensus_layer::height_index::Height;
use crate::consensus_layer::{artifacts::ConsensusMessage, pool_reader::PoolReader};
use crate::crypto::{CryptoHashOf, Signed};
use crate::SubnetParams;

use super::block_maker::Block;
use super::notary::{NotarizationShareContent, NotarizationShareContentCOD};

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
        FinalizationContent { height, block }
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
        let grouped_shares_separated_from_acks = aggregate(notarization_shares); // in case CoD is used, shares and acks for the same proposal are in two separate entries
                                                                                 // println!("Grouped shares separated from acks {:?}", grouped_shares_separated_from_acks);
        let grouped_shares = group_shares_and_acks(grouped_shares_separated_from_acks);
        // println!("Grouped shares: {:?}", grouped_shares);
        let notarizations = grouped_shares.into_iter().filter_map(|(notary_content, shares)| {
            let notary_content = match notary_content {
                NotarizationShareContent::COD(notary_content) => {
                    NotarizationContent {
                        height: notary_content.height,
                        block: notary_content.block
                    }
                }
                NotarizationShareContent::ICC(notary_content) => {
                    NotarizationContent {
                        height: notary_content.height,
                        block: notary_content.block
                    }
                }
            };
            if shares.len() >= (self.subnet_params.total_nodes_number - self.subnet_params.byzantine_nodes_number) as usize {
                if self.subnet_params.consensus_on_demand {
                    // println!("\nBlock with hash: {} received at least n-f notarization shares", notary_content.block.get_ref());
                    let block = get_block_by_hash_and_height(pool, &notary_content.block, notary_content.height);
                    // CoD rule 3c: notarize only 'good' blocks
                    match block_is_good(pool, &block.expect("block must be in pool")) {
                        true => {
                            println!("\nNotarization of block with hash: {} at height {} by committee: {:?}", notary_content.block.get_ref(), notary_content.height, shares);
                            Some(notary_content.clone())
                        },
                        false => {
                            None
                        }
                    }
                }
                else {
                    println!("\nNotarization of block with hash: {} at height {} by committee: {:?}", notary_content.block.get_ref(), notary_content.height, shares);
                    Some(notary_content)
                }
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
        }).collect();
        // println!("Notarizations: {:?}", notarizations);
        notarizations
    }

    /// Attempt to construct `Finalization`s
    fn aggregate_finalization_shares(&self, pool: &PoolReader<'_>) -> Vec<ConsensusMessage> {
        let finalization_shares = pool
            .get_finalization_shares(pool.get_finalized_height() + 1, pool.get_notarized_height());
        let grouped_shares = aggregate(finalization_shares);
        grouped_shares
            .into_iter()
            .filter_map(|(finalization_content, shares)| {
                if shares.len()
                    >= (self.subnet_params.total_nodes_number
                        - self.subnet_params.byzantine_nodes_number) as usize
                {
                    println!(
                        "\nFinalization of block with hash: {} at height {} by committee: {:?}",
                        finalization_content.block.get_ref(),
                        finalization_content.height,
                        shares
                    );
                    Some(finalization_content)
                } else {
                    None
                }
                .map(|finalization_content| {
                    ConsensusMessage::Finalization(Finalization {
                        content: FinalizationContent {
                            height: finalization_content.height,
                            block: finalization_content.block,
                        },
                        signature: 10, // committee signature
                    })
                })
            })
            .collect()
    }
}

pub fn aggregate<T: Ord>(
    shares: Box<dyn Iterator<Item = Signed<T, u8>>>,
) -> BTreeMap<T, BTreeSet<u8>> {
    shares.fold(
        BTreeMap::<T, BTreeSet<u8>>::new(),
        |mut grouped_shares, share| {
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
        },
    )
}

fn group_shares_and_acks(
    grouped_shares_separated_from_acks: BTreeMap<NotarizationShareContent, BTreeSet<u8>>,
) -> BTreeMap<NotarizationShareContent, BTreeSet<u8>> {
    // println!("\nGrouped shares separated from acks {:?}", grouped_shares_separated_from_acks);
    // we need to aggregate shares and acks for the same block proposal
    // if there are only acks for a proposal, we might still need to aggregate them into a notarization as
    // the acknowledger might not be able to create an FP-finalization even if it received n-p acks
    // this happens due to rule 2 of CoD which requires the parent of a block to be finalized in order for the block to be FP-finalized
    let grouped_shares_and_acks = grouped_shares_separated_from_acks.iter().fold(
        BTreeMap::<NotarizationShareContent, BTreeSet<u8>>::new(),
        |mut grouped_shares_and_acks, (notary_content, committee)| {
            match notary_content {
                NotarizationShareContent::COD(notary_content) => {
                    // here we only try to notarize blocks, therefore it is not important whether a notarization share is an acknowledgement or not
                    // we group all notarization shares (also acks) in one entry in order to count all the ones received for a block proposal
                    let generic_notary_content =
                        NotarizationShareContent::COD(NotarizationShareContentCOD {
                            is_ack: false, // set "is_ack" to false fopr each entry so that the acks are grouped with the shares for the same proposal
                            ..notary_content.clone()
                        });
                    match grouped_shares_and_acks.get_mut(&generic_notary_content) {
                        Some(grouped_by_proposal) => {
                            for share in committee {
                                grouped_by_proposal.insert(share.to_owned());
                            }
                        }
                        None => {
                            grouped_shares_and_acks
                                .insert(generic_notary_content.clone(), committee.clone());
                        }
                    }
                }
                // if only ICC is used, as there are no acks, there is no need to group them with the shares
                // shares for the same proposal are already aggregated by the "aggregate" function
                NotarizationShareContent::ICC(notary_content) => {
                    grouped_shares_and_acks.insert(
                        NotarizationShareContent::ICC(notary_content.clone()),
                        committee.clone(),
                    );
                }
            }
            grouped_shares_and_acks
        },
    );
    // println!("Grouped shares and acks {:?}", grouped_shares_and_acks);
    grouped_shares_and_acks
}
