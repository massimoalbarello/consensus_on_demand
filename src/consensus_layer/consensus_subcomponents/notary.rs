use serde::{Serialize, Deserialize};

use crate::{
    consensus_layer::{
        pool_reader::PoolReader, 
        artifacts::{ConsensusMessage, IntoInner}, 
        height_index::Height
    }, crypto::{Signed, Hashed, CryptoHashOf}
};

use super::block_maker::Block;

// NotarizationContent holds the values that are signed in a notarization
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct NotarizationContent {
    pub height: u64,
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


/// A notarization share is a multi-signature share on a notarization content.
/// If sufficiently many replicas create notarization shares, the shares can be
/// aggregated into a full notarization.
pub type NotarizationShare = Signed<NotarizationContent, u8>;

pub struct Notary {
    node_id: u8,
}

impl Notary {
    pub fn new(node_id: u8) -> Self {
        Self {
            node_id,
        }
    }

    pub fn on_state_change(&self, pool: &PoolReader<'_>) -> Vec<ConsensusMessage> {
        let mut notarization_shares = Vec::new();
        for (hash, artifact) in &pool.pool().validated().artifacts {
            println!("\n########## Notary ##########");
            match artifact.to_owned().into_inner() {
                ConsensusMessage::BlockProposal(proposal) => {
                    if !self.is_proposal_already_notarized_by_me(pool) {
                        if let Some(s) = self.notarize_block(pool, proposal) {
                            let notarization_share = ConsensusMessage::NotarizationShare(s);
                            println!("Notarization share: {:?}", notarization_share);
                            notarization_shares.push(notarization_share);
                        }
                    }
                },
                _ => (),
            }
        }
        notarization_shares
    }

    
    /// Return true if this node has already published a notarization share
    /// for the given block proposal. Return false otherwise.
    fn is_proposal_already_notarized_by_me(&self, pool: &PoolReader<'_>) -> bool {
        // if there is more than one artifact in the validated section of the pool it means that the node has already sent its notarization share
        pool.pool().validated().artifacts.len() > 1
    }

    /// Notarize and return a `NotarizationShare` for the given block
    fn notarize_block(
        &self,
        pool: &PoolReader<'_>,
        proposal: Signed<Hashed<Block>, u8>,
    ) -> Option<NotarizationShare> {
        let content = NotarizationContent::new(proposal.content.value.height, CryptoHashOf::from(proposal.content.hash));
        let signature = self.node_id;
        Some(NotarizationShare { content, signature })
    }
}