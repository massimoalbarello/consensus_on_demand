use serde::{Serialize, Deserialize};

use crate::consensus_layer::{pool_reader::PoolReader, artifacts::ConsensusMessage};
use super::block_maker::Block;

// NotarizationContent holds the values that are signed in a notarization
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct NotarizationContent {
    height: u64,
    block: Block
}

pub type NotarizationShare = NotarizationContent;

pub struct Notary {
    time: u64,
}

impl Notary {
    pub fn new() -> Self {
        Self {
            time: 0,
        }
    }

    pub fn on_state_change(&self, pool: &PoolReader<'_>) -> Vec<ConsensusMessage> {
        vec![]
    }
}