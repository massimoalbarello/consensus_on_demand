use std::collections::{BTreeMap, BTreeSet};

use serde::{Serialize, Deserialize};

use crate::{consensus_layer::{artifacts::ConsensusMessage, pool_reader::PoolReader, consensus_subcomponents::block_maker::Block}, crypto::CryptoHashOf};

use super::notary::NotarizationShareContent;


#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct GoodnessArtifact {
    parent: String,
    most_acks_child: String,
    most_acks_child_count: usize,
    total_acks_for_children: usize,
}

pub struct Goodifier {
    node_id: u8,
}

impl Goodifier {
    pub fn new(node_id: u8) -> Self {
        Self {
            node_id,
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
        println!("\n{:?}", grouped_acks);

        let children_goodness_artifacts: Vec<GoodnessArtifact> = grouped_acks.into_iter().map(|(parent, grouped_acks_by_parent)| {
            let mut children_goodness_artifact = GoodnessArtifact {
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
            children_goodness_artifact
        }).collect();
        for goodness_artifact in children_goodness_artifacts {
            println!("For parent {}, the child with most acks is {} and received {} acks out of {}", goodness_artifact.parent, goodness_artifact.most_acks_child, goodness_artifact.most_acks_child_count, goodness_artifact.total_acks_for_children);
        }
        vec![]
    }
}