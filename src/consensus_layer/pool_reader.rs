use crate::{
    consensus_layer::pool::ConsensusPoolImpl,
    crypto::CryptoHashOf
};

use super::{
    consensus_subcomponents::{
        notary::NotarizationShare, 
        block_maker::{Block, BlockProposal}
    },
    artifacts::ConsensusMessage, height_index::Height
};

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

    // Get max height of valid notarized blocks.
    pub fn get_notarized_height(&self) -> Height {
        let notarized_height = self.pool
            .validated()
            .notarization()
            .max_height();
        match notarized_height {
            Some(height) => {
                println!("Last block notarized at height: {}", height);
                height
            }
            None => {
                println!("No block notarized yet");
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
}