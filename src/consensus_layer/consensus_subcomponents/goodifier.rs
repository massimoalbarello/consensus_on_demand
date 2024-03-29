use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

use serde::{Deserialize, Serialize};

use crate::{
    consensus_layer::{artifacts::ConsensusMessage, height_index::Height, pool_reader::PoolReader},
    crypto::{CryptoHashOf, Hashed},
    time_source::{Time, TimeSource},
    SubnetParams,
};

use super::{
    block_maker::{Block, BlockProposal},
    notary::NotarizationShareContent,
};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct GoodnessArtifact {
    pub children_height: Height,
    pub parent_hash: String,
    pub most_acks_child: String,
    pub most_acks_child_count: usize,
    total_acks_for_children: usize,
    pub all_children_good: bool,
    pub timestamp: Time,
}

pub struct Goodifier {
    node_id: u8,
    subnet_params: SubnetParams,
    time_source: Arc<dyn TimeSource>,
}

impl Goodifier {
    pub fn new(node_id: u8, subnet_params: SubnetParams, time_source: Arc<dyn TimeSource>) -> Self {
        Self {
            node_id,
            subnet_params,
            time_source,
        }
    }

    pub fn on_state_change(&self, pool: &PoolReader<'_>) -> Vec<ConsensusMessage> {
        // println!("\n########## Goodifier ##########");
        let notarized_height = pool.get_notarized_height();
        let finalized_height = pool.get_finalized_height();
        // heights before the last finalized block do not need to be checked
        // check heights in which it is still possible for a goodness artifact to be updated
        (finalized_height..=notarized_height + 1)
            .filter_map(|h| {
                let consensus_messages_at_height = self.goodify_height(pool, h);
                match consensus_messages_at_height.len() {
                    0 => None,
                    _ => Some(consensus_messages_at_height),
                }
            })
            .flatten()
            .collect()
    }

    fn goodify_height(&self, pool: &PoolReader<'_>, h: Height) -> Vec<ConsensusMessage> {
        // group acks according to the parent of the block they are acknowledging
        // then for each parent group, group acks according to the block they are acknowledging
        let grouped_acks = pool.get_notarization_shares(h).fold(
            BTreeMap::<String, BTreeMap<String, BTreeSet<u8>>>::new(),
            |mut grouped_acks_by_parent, signed_share| {
                if let NotarizationShareContent::COD(notarization_share) = signed_share.content {
                    if notarization_share.is_ack {
                        let ack = notarization_share;
                        let signature = signed_share.signature;
                        match grouped_acks_by_parent.get_mut(&ack.block_parent_hash) {
                            Some(existing_parent_map) => {
                                match existing_parent_map.get_mut(ack.block.get_ref()) {
                                    Some(existing_block_set) => {
                                        existing_block_set.insert(signature);
                                    }
                                    None => {
                                        let mut block_set = BTreeSet::<u8>::new();
                                        let block_hash = ack.block.get_ref().clone();
                                        block_set.insert(signature);
                                        existing_parent_map.insert(block_hash, block_set);
                                    }
                                }
                            }
                            None => {
                                let mut grouped_acks_by_block =
                                    BTreeMap::<String, BTreeSet<u8>>::new();
                                let mut block_set = BTreeSet::<u8>::new();
                                let block_hash = ack.block.get_ref().clone();
                                let block_parent_hash = ack.block_parent_hash.clone();
                                block_set.insert(signature);
                                grouped_acks_by_block.insert(block_hash, block_set);
                                grouped_acks_by_parent
                                    .insert(block_parent_hash, grouped_acks_by_block);
                            }
                        };
                    }
                } else {
                    panic!("goodifier called while running original IC consensus");
                }
                grouped_acks_by_parent
            },
        );
        // println!("Grouped acks {:?}", grouped_acks);

        grouped_acks.into_iter().fold(
            Vec::new(),
            |mut goodness_consensus_messages_at_height, (parent_hash, grouped_acks_by_block)| {
                // initialize "goodness" artifact for a particular parent
                let mut children_goodness_artifact = GoodnessArtifact {
                    children_height: h,
                    parent_hash,
                    most_acks_child: String::from(""),
                    most_acks_child_count: 0,
                    total_acks_for_children: 0,
                    all_children_good: false,
                    timestamp: self.time_source.get_relative_time(),
                };

                // count total number of acks on children and determine which child is the one with the most acks
                for (block_hash, acks_for_block) in grouped_acks_by_block {
                    let acks_for_current_block_count = acks_for_block.len();
                    if acks_for_current_block_count
                        > children_goodness_artifact.most_acks_child_count
                    {
                        children_goodness_artifact.most_acks_child = block_hash.clone();
                        children_goodness_artifact.most_acks_child_count =
                            acks_for_current_block_count;
                    }
                    children_goodness_artifact.total_acks_for_children +=
                        acks_for_current_block_count;
                }

                // for each parent, check conditions to determine which children are "good"
                match pool.get_latest_goodness_artifact_for_parent(
                    &children_goodness_artifact.parent_hash,
                    h,
                ) {
                    // if "goodness" artifact does not exist, we check whether it can be created according to currently received acks
                    None => {
                        if children_goodness_artifact.total_acks_for_children
                            - children_goodness_artifact.most_acks_child_count
                            > (self.subnet_params.byzantine_nodes_number
                                + self.subnet_params.disagreeing_nodes_number)
                                as usize
                        {
                            // println!("\nAll children of: {} at height: {} are good", children_goodness_artifact.parent_hash, h);
                            children_goodness_artifact.all_children_good = true;
                            goodness_consensus_messages_at_height.push(
                                ConsensusMessage::GoodnessArtifact(
                                    children_goodness_artifact.clone(),
                                ),
                            );
                        } else if children_goodness_artifact.total_acks_for_children
                            >= (self.subnet_params.total_nodes_number
                                - self.subnet_params.byzantine_nodes_number)
                                as usize
                        {
                            // println!("\nFor parent: {} at height: {}, the good child with most acks is: {} and received: {} acks out of: {}", children_goodness_artifact.parent_hash, children_goodness_artifact.children_height-1, children_goodness_artifact.most_acks_child, children_goodness_artifact.most_acks_child_count, children_goodness_artifact.total_acks_for_children);
                            goodness_consensus_messages_at_height.push(
                                ConsensusMessage::GoodnessArtifact(
                                    children_goodness_artifact.clone(),
                                ),
                            );
                        }
                    }
                    // if the "goodness" artifact already exists, we must check whether it should be updated
                    Some(previous_goodness_artifact) => {
                        // if all children are "good", the "goodness" artifact for this parent does not have to be updated as all children will remain "good"
                        // and in this case we do not care about which one is the one with the most acks
                        if !previous_goodness_artifact.all_children_good {
                            // if all children become "good" we create an updated "goodness" artifact
                            if children_goodness_artifact.total_acks_for_children
                                - children_goodness_artifact.most_acks_child_count
                                > (self.subnet_params.byzantine_nodes_number
                                    + self.subnet_params.disagreeing_nodes_number)
                                    as usize
                            {
                                // println!("\nAll children of: {} at height: {} are good", children_goodness_artifact.parent_hash, h);
                                children_goodness_artifact.all_children_good = true;
                                goodness_consensus_messages_at_height.push(
                                    ConsensusMessage::GoodnessArtifact(
                                        children_goodness_artifact.clone(),
                                    ),
                                );
                            } else if children_goodness_artifact.total_acks_for_children
                                >= (self.subnet_params.total_nodes_number
                                    - self.subnet_params.byzantine_nodes_number)
                                    as usize
                            {
                                // if the child with most acks is different from the one stored in the previous "goodness" artifact and has more acks
                                // we create an updated "goodness" child
                                if previous_goodness_artifact.most_acks_child
                                    != children_goodness_artifact.most_acks_child
                                    && previous_goodness_artifact.most_acks_child_count
                                        < children_goodness_artifact.most_acks_child_count
                                {
                                    // println!("\nUpdating good child with most acks: {} for parent: {} at height: {}", children_goodness_artifact.most_acks_child, children_goodness_artifact.parent_hash, children_goodness_artifact.children_height-1);
                                    goodness_consensus_messages_at_height.push(
                                        ConsensusMessage::GoodnessArtifact(
                                            children_goodness_artifact.clone(),
                                        ),
                                    );
                                }
                            }
                        }
                    }
                };

                goodness_consensus_messages_at_height
            },
        )
    }
}

pub fn get_block_by_hash_and_height(
    pool: &PoolReader<'_>,
    hash: &CryptoHashOf<Block>,
    h: Height,
) -> Option<Block> {
    // return a valid block with the matching hash and height if it exists.
    let mut blocks: Vec<BlockProposal> = pool
        .pool()
        .validated()
        .block_proposal()
        .get_by_height(h)
        .filter(|x| x.content.get_hash() == hash.get_ref())
        .collect();
    match blocks.len() {
        1 => Some(blocks.remove(0).content.value),
        _ => None,
    }
}

pub fn block_is_good(pool: &PoolReader<'_>, block: &Block) -> bool {
    // block is one of the children for the latest "goodness" artifact
    // pool.print_goodness_artifacts_at_height(block.height);
    match pool.get_latest_goodness_artifact_for_parent(&block.parent, block.height) {
        Some(goodness_artifact) => {
            // println!("\nLatest goodness artifact {:?}", goodness_artifact);
            if goodness_artifact.all_children_good {
                return true;
            }
            let block_hash = Hashed::crypto_hash(&block);
            // println!("Block to be checked: {}", block_hash);
            goodness_artifact.most_acks_child == block_hash
        }
        None => {
            if block.height == 0 {
                return true; // genesis is good
            }
            false
        }
    }
}
