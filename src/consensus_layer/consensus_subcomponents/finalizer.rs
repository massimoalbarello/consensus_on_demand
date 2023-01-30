use std::cell::RefCell;

use serde::{Deserialize, Serialize};

use crate::{
    consensus_layer::{artifacts::ConsensusMessage, height_index::Height, pool_reader::PoolReader},
    crypto::{CryptoHashOf, Hashed, Signed},
    SubnetParams,
};

use super::{block_maker::Block, goodifier::block_is_good, notary::NotarizationShareContent};

/// FinalizationShareContent holds the values that are signed in a finalization share
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct FinalizationShareContent {
    pub height: Height,
    pub block: CryptoHashOf<Block>,
}

impl FinalizationShareContent {
    pub fn new(height: Height, block: CryptoHashOf<Block>) -> Self {
        FinalizationShareContent { height, block }
    }
}

/// A finalization share is a multi-signature share on a finalization content.
/// If sufficiently many replicas create finalization shares, the shares can be
/// aggregated into a full finalization.
pub type FinalizationShare = Signed<FinalizationShareContent, u8>;

pub struct Finalizer {
    node_id: u8,
    subnet_params: SubnetParams,
    prev_finalized_height: RefCell<Height>,
}

impl Finalizer {
    #[allow(clippy::too_many_arguments)]
    pub fn new(node_id: u8, subnet_params: SubnetParams) -> Self {
        Self {
            node_id,
            subnet_params,
            prev_finalized_height: RefCell::new(0),
        }
    }

    /// Attempt to:
    /// * deliver finalized blocks (as `Batch`s) via `Messaging`
    /// * publish finalization shares for relevant rounds
    pub fn on_state_change(&self, pool: &PoolReader<'_>) -> Vec<ConsensusMessage> {
        // println!("\n########## Finalizer ##########");
        let notarized_height = pool.get_notarized_height();
        let finalized_height = pool.get_finalized_height();

        if *self.prev_finalized_height.borrow() < finalized_height {
            *self.prev_finalized_height.borrow_mut() = finalized_height;
        }

        // Try to finalize rounds from finalized_height + 1 up to (and including) notarized_height
        // if received a finalization for a block at height h+1 before a notarization for the same height
        // in rounds in which the CoD fast path is used, the range will be empty and thus no finalization shares will be created
        (finalized_height + 1..=notarized_height)
            .filter_map(|h| match self.finalize_height(pool, h) {
                Some(f) => {
                    let finalization_share = ConsensusMessage::FinalizationShare(f);
                    println!("\nCreated finalization share: {:?}", finalization_share);
                    Some(finalization_share)
                }
                None => None,
            })
            .collect()
    }

    /// Try to create a finalization share for a notarized block at the given
    /// height
    fn finalize_height(&self, pool: &PoolReader<'_>, height: Height) -> Option<FinalizationShare> {
        let content = FinalizationShareContent::new(
            height,
            CryptoHashOf::new(Hashed::crypto_hash(
                &self.pick_block_to_finality_sign(pool, height)?,
            )),
        );
        // add 10 to make the hash of the finalization share different from the one of the notarization share
        let signature = 10 + self.node_id;
        Some(FinalizationShare { content, signature })
    }

    /// Attempt to find a notarized block at the given height that this node
    /// can publish a finalization share for. A block is only returned if:
    /// * This replica has not created a finalization share for height `h` yet
    /// * This replica has exactly one fully notarized block at height `h`
    /// * This replica has not created a notarization share for height `h` on
    ///   any block other than the single fully notarized block at height `h`
    ///
    /// In this case, the the single notarized block is returned. Otherwise,
    /// return `None`
    fn pick_block_to_finality_sign(&self, pool: &PoolReader<'_>, h: Height) -> Option<Block> {
        // if this replica already created a finalization share for height `h`, we do
        // not need to finality sign a block anymore
        if pool
            .get_finalization_shares(h, h)
            .any(|share| share.signature == 10 + self.node_id)
        {
            return None;
        }

        // look up all fully notarized blocks for height `h`
        let mut notarized_blocks: Vec<_> = pool.get_notarized_blocks(h).collect();

        // Check if we have exactly one notarized block, and if so, determine that block
        let notarized_block = match notarized_blocks.len() {
            0 => {
                // If there are no notarized blocks at height `h`, we panic, as we should only
                // try to finalize heights that are notarized.
                panic!(
                    "Trying to finalize height {:?} but no notarized block found",
                    h
                );
            }
            1 => notarized_blocks.remove(0),
            _ => {
                // if there are multiple fully notarized blocks, there is no chance we reach
                // finality, so there is no point in creating a finalization share
                return None;
            }
        };

        if self.subnet_params.consensus_on_demand {
            // CoD rule 3b: send finalization share only for "good" block
            if !block_is_good(pool, &notarized_block) {
                return None;
            }
        }

        // If notarization shares exists created by this replica at height `h`
        // that sign a block different than `notarized_block`, do not finalize.
        let other_notarized_shares_exists =
            pool.get_notarization_shares(h).any(|x| match x.content {
                NotarizationShareContent::COD(share_content) => {
                    x.signature == self.node_id
                        && share_content.block
                            != CryptoHashOf::new(Hashed::crypto_hash(&notarized_block))
                }
                NotarizationShareContent::ICC(share_content) => {
                    x.signature == self.node_id
                        && share_content.block
                            != CryptoHashOf::new(Hashed::crypto_hash(&notarized_block))
                }
            });
        if other_notarized_shares_exists {
            return None;
        }

        Some(notarized_block)
    }
}
