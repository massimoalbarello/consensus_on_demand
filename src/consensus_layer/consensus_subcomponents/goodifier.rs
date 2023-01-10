use std::collections::{BTreeMap, BTreeSet};

use serde::{Serialize, Deserialize};

use crate::{consensus_layer::{artifacts::ConsensusMessage, pool_reader::PoolReader, height_index::Height}, SubnetParams};

use super::notary::NotarizationShareContent;


#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct GoodnessArtifact {
    pub height: Height,
    pub parent: String,
    most_acks_child: String,
    most_acks_child_count: usize,
    total_acks_for_children: usize,
}

pub struct Goodifier {
    node_id: u8,
    subnet_params: SubnetParams,
}

impl Goodifier {
    pub fn new(node_id: u8, subnet_params: SubnetParams) -> Self {
        Self {
            node_id,
            subnet_params,
        }
    }

    pub fn on_state_change(&self, pool: &PoolReader<'_>) -> Vec<ConsensusMessage> {
        let height = pool.get_notarized_height() + 1;
        let grouped_acks = pool
        .get_notarization_shares(height)
        .fold(BTreeMap::<String, BTreeMap<String, BTreeSet<u8>>>::new(), |mut grouped_acks_by_parent, signed_share| {
            if let NotarizationShareContent::COD(notarization_share) = signed_share.content {
                if notarization_share.is_ack {
                    let ack = notarization_share;
                    let signature = signed_share.signature;
                    match grouped_acks_by_parent.get_mut(&ack.block_parent_hash) {
                        Some(existing_parent_map) => {
                            match existing_parent_map.get_mut(ack.block.get_ref()) {
                                Some(existing_block_set) => {
                                    existing_block_set.insert(signature);
                                },
                                None => {
                                    let mut block_set = BTreeSet::<u8>::new();
                                    let block_hash = ack.block.get_ref().clone();
                                    block_set.insert(signature);
                                    existing_parent_map.insert(block_hash, block_set);
                                }
                            }
                        },
                        None => {
                            let mut grouped_acks_by_block = BTreeMap::<String, BTreeSet<u8>>::new();
                            let mut block_set = BTreeSet::<u8>::new();
                            let block_hash = ack.block.get_ref().clone();
                            let block_parent_hash = ack.block_parent_hash.clone();
                            block_set.insert(signature);
                            grouped_acks_by_block.insert(block_hash, block_set);
                            grouped_acks_by_parent.insert(block_parent_hash, grouped_acks_by_block);
                        }
                    };
                }
            }
            else {
                panic!("component called in original IC consensus")
            }
            grouped_acks_by_parent
        });

        grouped_acks.into_iter().filter_map(|(parent, grouped_acks_by_parent)| {
            let mut children_goodness_artifact = GoodnessArtifact {
                height,
                parent,
                most_acks_child: String::from(""),
                most_acks_child_count: 0,
                total_acks_for_children: 0,
            };
            for (block, grouped_acks_by_block) in grouped_acks_by_parent {
                let acks_for_current_block_count = grouped_acks_by_block.len();
                if acks_for_current_block_count > children_goodness_artifact.most_acks_child_count {
                    children_goodness_artifact.most_acks_child = block.clone();
                    children_goodness_artifact.most_acks_child_count = acks_for_current_block_count;
                    children_goodness_artifact.total_acks_for_children += acks_for_current_block_count;
                }
            }
            if !pool.exists_goodness_artifact_for_parent(&children_goodness_artifact.parent, height) {
                if children_goodness_artifact.total_acks_for_children - children_goodness_artifact.most_acks_child_count > (self.subnet_params.byzantine_nodes_number + self.subnet_params.disagreeing_nodes_number) as usize {
                    println!("\n!!!!!!!!!!!!!!! All children of {} are GOOD !!!!!!!!!!!!!!!", children_goodness_artifact.parent);
                    return Some(ConsensusMessage::GoodnessArtifact(children_goodness_artifact));
                }
                else if children_goodness_artifact.total_acks_for_children > (self.subnet_params.total_nodes_number - self.subnet_params.byzantine_nodes_number) as usize {
                    println!("\nFor parent {}, the good child with most acks is {} and received {} acks out of {}", children_goodness_artifact.parent, children_goodness_artifact.most_acks_child, children_goodness_artifact.most_acks_child_count, children_goodness_artifact.total_acks_for_children);
                    return Some(ConsensusMessage::GoodnessArtifact(children_goodness_artifact));
                }
                else {
                    return None;
                }
            }
            else {
                return None;
            }
        }).collect()
    }
}