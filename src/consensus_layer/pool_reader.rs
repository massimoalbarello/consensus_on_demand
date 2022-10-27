use crate::{
    consensus_layer::pool::ConsensusPoolImpl,
    crypto::CryptoHashOf, time_source::Time
};

use super::{
    consensus_subcomponents::{
        notary::NotarizationShare, 
        block_maker::{Block, BlockProposal}
    },
    height_index::Height, artifacts::ConsensusMessageHashable
};

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
        h: Height,
    ) -> Box<dyn Iterator<Item = NotarizationShare>> {
        self.pool.validated().notarization_share().get_by_height(h)
    }

    // Get max height of valid notarized blocks.
    pub fn get_notarized_height(&self) -> Height {
        let notarized_height = self.pool
            .validated()
            .notarization()
            .max_height();
        match notarized_height {
            Some(height) => {
                height
            }
            None => {
                0
            }
        }
    }

    /// Return a valid block with the matching hash and height if it exists.
    pub fn get_block(&self, hash: &CryptoHashOf<Block>, h: Height) -> Result<Block, ()> {
        let mut blocks: Vec<BlockProposal> = self
            .pool
            .validated()
            .block_proposal()
            .get_by_height(h)
            .filter(|x| x.content.get_hash() == hash.get_ref())
            .collect();
        match blocks.len() {
            1 => Ok(blocks.remove(0).content.value),
            _ => Err(()),
        }
    }

    /// Return all valid notarized blocks of a given height.
    pub fn get_notarized_blocks(&'a self, h: Height) -> Box<dyn Iterator<Item = Block> + 'a> {
        Box::new(
            self.pool
                .validated()
                .notarization()
                .get_by_height(h)
                .map(move |x| self.get_block(&x.content.block, h).unwrap()),
        )
    }

    
    /// Get the round start time of a given height, which is the max timestamp
    /// of first notarization and random beacon of the previous height.
    /// Return None if a timestamp is not found.
    pub fn get_round_start_time(&self, height: Height) -> Option<Time> {
        let validated = self.pool.validated();

        let get_notarization_time = |h| {
            validated
                .notarization()
                .get_by_height(h)
                .flat_map(|x| validated.get_timestamp(&x.get_id()))
                .min()
        };
        let prev_height = height - 1;
        let notarization_time = get_notarization_time(prev_height)
            .map(|notarization_time| notarization_time);
        println!("Last notarization time: {:?}", notarization_time);
        notarization_time
    }
}