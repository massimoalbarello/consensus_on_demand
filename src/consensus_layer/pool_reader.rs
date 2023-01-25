use crate::{
    consensus_layer::pool::ConsensusPoolImpl,
    crypto::CryptoHashOf, time_source::Time
};

use super::{
    consensus_subcomponents::{
        notary::{NotarizationShare, NotarizationShareContent}, 
        block_maker::{Block, BlockProposal}, finalizer::FinalizationShare, goodifier::GoodnessArtifact
    },
    height_index::{Height, HeightRange}, artifacts::ConsensusMessageHashable
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

    pub fn count_acknowledgements_at_height(
        &self,
        h: Height,
    ) -> usize {
        self.get_notarization_shares(h)
            .filter(|share| if let NotarizationShareContent::COD(_) = share.content { true } else { false })
            .count()
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

    /// Get all valid finalization shares in the given height range, inclusive.
    pub fn get_finalization_shares(
        &self,
        from: Height,
        to: Height,
    ) -> Box<dyn Iterator<Item = FinalizationShare>> {
        self.pool
            .validated()
            .finalization_share()
            .get_by_height_range(HeightRange::new(from, to))
    }

    /// Get max height of valid finalized blocks.
    pub fn get_finalized_height(&self) -> Height {
        match self.get_finalized_tip() {
            Some(block) => block.height,
            None => {
                0
            }
        }
    }

    /// Get the finalized block with greatest height.
    pub fn get_finalized_tip(&self) -> Option<Block> {
        self.pool.finalized_block()
    }

    pub fn get_finalized_block_hash_at_height(&self, height: Height) -> Option<String> {
        self.pool.finalized_block_hash_at_height(height)
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

    pub fn print_goodness_artifacts_at_height(&self, height: Height) {
        for good in self.pool
            .validated()
            .goodness_artifact()
            .get_by_height(height)
        {
            println!("{:?}", good);
        }
    }

    pub fn get_goodness_height(&self) -> Height {
        match self.pool
            .validated()
            .goodness_artifact()
            .max_height()
        {
            None => 0,
            Some(height) => height 
        }
    }

    pub fn get_latest_goodness_artifact_for_parent(&self, parent_hash: &String, children_height: Height) -> Option<GoodnessArtifact> {
        self.pool
            .validated()
            .goodness_artifact()
            .get_by_height(children_height)
            .filter(|goodness_artifact| goodness_artifact.parent_hash.eq(parent_hash))
            .max_by(|first, second| first.timestamp.cmp(&second.timestamp))
    }

    pub fn exists_goodness_artifact_for_parent(&self, parent_hash: &String, height: Height) -> bool {
        match self.get_latest_goodness_artifact_for_parent(parent_hash, height) {
            Some(_) => true,
            None => false,
        }
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
        get_notarization_time(prev_height)
            .map(|notarization_time| notarization_time)
    }
}