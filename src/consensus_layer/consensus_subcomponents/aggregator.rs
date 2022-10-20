//! The share aggregator is responsible for the aggregation of different types
//! of shares into full objects. That is, it constructs Random Beacon objects
//! from random beacon shares, Notarizations from notarization shares and
//! Finalizations from finalization shares.

use serde::{Deserialize, Serialize};

use crate::consensus_layer::{
    artifacts::N,
    pool_reader::PoolReader,
    artifacts::ConsensusMessage,
};
use crate::crypto::Signed;


// NotarizationContent holds the values that are signed in a notarization
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct NotarizationContent {
    pub height: u64,
    pub block: String,
}

impl NotarizationContent {
    pub fn new(block_height: u64, block_hash: String) -> Self {
        Self {
            height: block_height,
            block: block_hash,
        }
    }
}

pub type Notarization = Signed<NotarizationContent, u8>;

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
        messages
    }

    /// Attempt to construct `Notarization`s at `notarized_height + 1`
    fn aggregate_notarization_shares(&self, pool: &PoolReader<'_>) -> Vec<ConsensusMessage> {
        let notarization_shares = pool.get_notarization_shares();
        let mut notarizations  = vec![]; 
        let notarization_hash = String::from("Notarization hash");
        if notarization_shares.len() >= N-1 && !pool.pool().validated().artifacts.contains_key(&notarization_hash) {
            let content = NotarizationContent::new(notarization_shares[0].content.height, notarization_hash);
            let signature = self.node_id;
            notarizations.push(ConsensusMessage::Notarization(Notarization { content, signature }))
        }
        notarizations
    }
}