use serde::{Serialize, Deserialize};

use crate::{consensus_layer::{pool_reader::PoolReader, artifacts::{ConsensusMessage, IntoInner}}, crypto::{Signed, Hashed}};

// NotarizationContent holds the values that are signed in a notarization
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct NotarizationContent {
    pub height: u64,
    pub block: String,
}

impl NotarizationContent {
    fn new(block_height: u64, block_hash: String) -> Self {
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
                ConsensusMessage::NotarizationShare(_) => (),
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
        proposal: Signed<Hashed, u8>,
    ) -> Option<NotarizationShare> {
        // concatenating the node id in order to distinguish the locally generated notarization share from the ones received from peers in the artifacts pool
        let block_hash = format!("{}{}", proposal.content.hash, self.node_id.to_string());
        let content = NotarizationContent::new(proposal.content.value.height, block_hash);
        let signature = self.node_id;
        Some(NotarizationShare { content, signature })
    }
}