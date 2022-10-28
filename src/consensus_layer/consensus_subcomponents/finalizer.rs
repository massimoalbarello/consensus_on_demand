use std::cell::RefCell;

use serde::{Serialize, Deserialize};

use crate::{consensus_layer::{height_index::Height, pool_reader::PoolReader, artifacts::ConsensusMessage}, crypto::{CryptoHashOf, Signed}};

use super::block_maker::Block;


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

/// A finalization share is a multi-signature share on a finalization content.
/// If sufficiently many replicas create finalization shares, the shares can be
/// aggregated into a full finalization.
pub type FinalizationShare = Signed<FinalizationContent, u8>;

pub struct Finalizer {
    prev_finalized_height: RefCell<Height>,
}

impl Finalizer {
    #[allow(clippy::too_many_arguments)]
    pub fn new() -> Self {
        Self {
            prev_finalized_height: RefCell::new(0),
        }
    }

    /// Attempt to:
    /// * deliver finalized blocks (as `Batch`s) via `Messaging`
    /// * publish finalization shares for relevant rounds
    pub fn on_state_change(&self, pool: &PoolReader<'_>) -> Vec<ConsensusMessage> {
        let notarized_height = pool.get_notarized_height();
        let finalized_height = pool.get_finalized_height();
        println!("Finalized height: {}", finalized_height);

        vec![]
    }
}
