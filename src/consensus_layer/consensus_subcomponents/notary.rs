use std::{sync::Arc, time::Duration};

use serde::{Serialize, Deserialize};

use crate::{
    consensus_layer::{
        pool_reader::PoolReader, 
        artifacts::ConsensusMessage, 
        height_index::Height
    }, crypto::{Signed, Hashed, CryptoHashOf}, time_source::TimeSource
};

use super::block_maker::{Block, BlockProposal};

pub const NOTARIZATION_DELAY_UNIT: Duration = Duration::from_millis(5000);

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
    time_source: Arc<dyn TimeSource>,
}

impl Notary {
    pub fn new(node_id: u8, time_source: Arc<dyn TimeSource>) -> Self {
        Self {
            node_id,
            time_source,
        }
    }

    pub fn on_state_change(&self, pool: &PoolReader<'_>) -> Vec<ConsensusMessage> {
        let notarized_height = pool.get_notarized_height();
        let mut notarization_shares = Vec::new();
        let height = notarized_height + 1;
        for proposal in find_lowest_ranked_proposals(pool, height) {
            if !self.is_proposal_already_notarized_by_me(pool, &proposal) {
                if let Some(s) = self.notarize_block(pool, proposal) {
                    println!("\n########## Notary ##########");
                    println!("Created notarization share: {:?}", s);
                    notarization_shares.push(ConsensusMessage::NotarizationShare(s));
                }
            }
        }
        notarization_shares
    }

    
    /// Return true if this node has already published a notarization share
    /// for the given block proposal. Return false otherwise.
    fn is_proposal_already_notarized_by_me<'a>(
        &self,
        pool: &PoolReader<'_>,
        proposal: &'a BlockProposal,
    ) -> bool {
        let height = proposal.content.value.height;
        pool.get_notarization_shares(height)
            .filter(|s| s.signature == self.node_id)
            .any(|s| proposal.content.hash.eq(s.content.block.get_ref()))
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

/// Return the validated block proposals with the lowest rank at height `h`, if
/// there are any. Else return `None`.
pub fn find_lowest_ranked_proposals(pool: &PoolReader<'_>, h: Height) -> Vec<BlockProposal> {
    let (_, best_proposals) = pool
        .pool()
        .validated()
        .block_proposal()
        .get_by_height(h)
        .fold(
            (None, Vec::new()),
            |(mut best_rank, mut best_proposals), proposal| {
                if best_rank.is_none() || best_rank.unwrap() > proposal.content.value.rank {
                    best_rank = Some(proposal.content.value.rank);
                    best_proposals = vec![proposal];
                } else if Some(proposal.content.value.rank) == best_rank {
                    best_proposals.push(proposal);
                }
                (best_rank, best_proposals)
            },
        );
    best_proposals
}