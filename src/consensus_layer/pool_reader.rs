use crate::consensus_layer::pool::ConsensusPoolImpl;

use super::{consensus_subcomponents::notary::NotarizationShare, artifacts::ConsensusMessage};
use crate::consensus_layer::artifacts::IntoInner;

// A struct and corresponding impl with helper methods to obtain particular
// artifacts/messages from the artifact pool.
pub struct PoolReader<'a> {
    pool: &'a ConsensusPoolImpl,
}

impl<'a> PoolReader<'a> {
    // Create a PoolReader for a ConsensusPool.
    pub fn new(pool: &'a ConsensusPoolImpl) -> Self {
        Self {
            pool,
        }
    }

    /// Get the underlying pool.
    pub fn pool(&self) -> &'a ConsensusPoolImpl {
        self.pool
    }

    
    /// Get all valid notarization shares at the given height.
    pub fn get_notarization_shares(
        &self,
    ) -> Vec<NotarizationShare> {
        let mut shares = vec![];
        for (_, artifact) in &self.pool().validated().artifacts {
            match artifact.to_owned().into_inner() {
                ConsensusMessage::NotarizationShare(share) => shares.push(share),
                _ => (),
            }
        }
        shares
    }
}