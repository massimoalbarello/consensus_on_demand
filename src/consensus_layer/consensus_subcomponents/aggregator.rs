//! The share aggregator is responsible for the aggregation of different types
//! of shares into full objects. That is, it constructs Random Beacon objects
//! from random beacon shares, Notarizations from notarization shares and
//! Finalizations from finalization shares.

use std::collections::{BTreeMap, BTreeSet};
use serde::{Deserialize, Serialize};

use crate::consensus_layer::artifacts::N;
use crate::consensus_layer::height_index::Height;
use crate::consensus_layer::{
    pool_reader::PoolReader,
    artifacts::ConsensusMessage,
};
use crate::crypto::{Signed, CryptoHashOf};

use super::block_maker::Block;
use super::notary;


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
}

impl ShareAggregator {
    pub fn new(node_id: u8) -> Self {
        Self {
            node_id,
        }
    }

    /// Attempt to construct artifacts from artifact shares in the artifact
    /// pool
    pub fn on_state_change(&self, pool: &PoolReader<'_>) -> Vec<ConsensusMessage> {
        let mut messages = Vec::new();
        messages.append(&mut self.aggregate_notarization_shares(pool));
        messages.append(&mut self.aggregate_finalization_shares(pool));
        messages
    }

    /// Attempt to construct `Notarization`s at `notarized_height + 1`
    fn aggregate_notarization_shares(&self, pool: &PoolReader<'_>) -> Vec<ConsensusMessage> {
        let height = pool.get_notarized_height() + 1;
        let notarization_shares = pool.get_notarization_shares(height);
        let grouped_shares = aggregate(notarization_shares);
        grouped_shares.into_iter().filter_map(|(notary_content, shares)| {
            if shares.len() >= N-1 {
                println!("\n########## Aggregator ##########");
                println!("Notarization of block with hash: {} at height {} by committee: {:?}", notary_content.block.get_ref(), notary_content.height, shares);
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
            if shares.len() >= N-1 {
                println!("\n########## Aggregator ##########");
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

fn aggregate<T: Ord>(shares: Box<dyn Iterator<Item = Signed<T, u8>>>) -> BTreeMap<T, BTreeSet<u8>>{
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