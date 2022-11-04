use std::{sync::Arc, time::Duration};

use serde::{Serialize, Deserialize};

use crate::{consensus_layer::{
    pool_reader::PoolReader,
    artifacts::{ConsensusMessage, N}, height_index::Height
}, crypto::{Signed, Hashed}, time_source::TimeSource};

use super::notary::NOTARIZATION_UNIT_DELAY;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub struct Payload {}

impl Payload {
    pub fn new() -> Self {
        Self {}
    }
}

// Block is the type that is used to create blocks out of which we build a
// block chain
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct Block {
    // the parent block that this block extends, forming a block chain
    parent: String,
    // the payload of the block
    payload: Payload,
    // the height of the block, which is the height of the parent + 1
    pub height: u64,
    // rank indicates the rank of the block maker that created this block
    pub rank: u8,
}

impl Block {
    // Create a new block
    pub fn new(
        parent: String,
        payload: Payload,
        height: u64,
        rank: u8,
    ) -> Self {
        Block {
            parent,
            payload,
            height,
            rank,
        }
    }
}

/// HashedBlock contains a Block together with its hash
pub type HashedBlock = Hashed<Block>;

pub type BlockProposal = Signed<HashedBlock, u8>;

pub struct RandomBeacon {}

pub struct BlockMaker {
    node_id: u8,
    time_source: Arc<dyn TimeSource>,
}

impl BlockMaker {
    pub fn new(node_id: u8, time_source: Arc<dyn TimeSource>) -> Self {
        Self {
            node_id,
            time_source,
        }
    }

    pub fn on_state_change(&self, pool: &PoolReader<'_>) -> Option<ConsensusMessage> {
        println!("\n########## Block maker ##########");
        let my_node_id = self.node_id;
        let (beacon, parent) = get_dependencies(pool).unwrap();
        let height: u64 = parent.height + 1;
        match get_block_maker_rank(height, &beacon, my_node_id)
        {
            rank => {
                if !already_proposed(pool, height, my_node_id)
                    && !self.is_better_block_proposal_available(pool, height, rank)
                    && is_time_to_make_block(
                        pool,
                        height,
                        rank,
                        self.time_source.as_ref(),
                        my_node_id
                    )
                {
                    let block_proposal = self.propose_block(pool, rank, parent).map(|proposal| {
                        ConsensusMessage::BlockProposal(proposal)
                    });
                    println!("Created block proposal: {:?}", block_proposal);
                    block_proposal
                }
                else {
                    None
                }
            }
        }
    }

    /// Return true if the validated pool contains a better (lower ranked) block
    /// proposal than the given rank, for the given height.
    fn is_better_block_proposal_available(
        &self,
        pool: &PoolReader<'_>,
        height: Height,
        rank: u8,
    ) -> bool {
        if let Some(block) = find_lowest_ranked_proposals(pool, height).first() {
            return block.content.value.rank < rank;
        }
        false
    }

    // Construct a block proposal
    fn propose_block(
        &self,
        pool: &PoolReader<'_>,
        rank: u8,
        parent: Block,
    ) -> Option<BlockProposal> {
        let parent_hash = Hashed::crypto_hash(&parent);
        let height: u64 = parent.height + 1;
        self.construct_block_proposal(
            pool,
            parent,
            parent_hash,
            height,
            rank,
        )
    }

    // Construct a block proposal with specified validation context, parent
    // block, rank, and batch payload. This function completes the block by
    // adding a DKG payload and signs the block to obtain a block proposal.
    #[allow(clippy::too_many_arguments)]
    fn construct_block_proposal(
        &self,
        pool: &PoolReader<'_>,
        parent: Block,
        parent_hash: String,
        height: u64,
        rank: u8,
    ) -> Option<BlockProposal> {
        let payload = Payload::new();
        let block = Block::new(parent_hash, payload, height, rank);
        Some(BlockProposal {
            signature: self.node_id,
            content: Hashed::new(block),
        })
    }
}


// Return the parent random beacon and block of the latest round for which
// this node might propose a block.
// Return None otherwise.
fn get_dependencies(pool: &PoolReader<'_>) -> Option<(RandomBeacon, Block)> {
    let notarized_height = pool.get_notarized_height();
    // println!("Last block notarized at height: {}", notarized_height);
    let parent = pool
        .get_notarized_blocks(notarized_height)
        .min_by(|block1, block2| block1.rank.cmp(&block2.rank));
    match parent {
        Some(parent) => {
            // println!("Parent block: {:?}", parent);
            Some((
                RandomBeacon {},
                parent
            ))
        },
        None => {
            Some((
                RandomBeacon {},
                Block {
                    parent: String::from("Genesis has no parent"),
                    payload: Payload::new(),
                    height: 0,
                    rank: 0,
                }
            ))
        }
    }
}

fn get_block_maker_rank(height: u64, beacon: &RandomBeacon, my_node_id: u8) -> u8 {
    let rank = ((height + my_node_id as u64 - 2) % N as u64) as u8;
    // println!("Local rank for height {} is: {}", height, rank);
    rank
}

// Return true if this node has already made a proposal at the given height.
fn already_proposed(pool: &PoolReader<'_>, h: u64, this_node: u8) -> bool {
    pool.pool()
        .validated()
        .block_proposal()
        .get_by_height(h)
        .any(|p| p.signature == this_node)
}

// Return true if the time since round start is greater than the required block
// maker delay for the given rank.
fn is_time_to_make_block(
    pool: &PoolReader<'_>,
    height: u64,
    rank: u8,
    time_source: &dyn TimeSource,
    node_id: u8
) -> bool {
    let block_maker_delay =
    match get_block_maker_delay(rank) {
        Some(delay) => delay,
        _ => return false,
    };
    match pool.get_round_start_time(height) {
        Some(start_time) => {
            let current_time = time_source.get_relative_time();
            if current_time >= start_time + block_maker_delay {
                return true
            }
            false
        }
        None => {
            // if there is no previous notarization, node 1 proposes the first block (has rank 0 in the first round)
            if node_id == 1 && rank == 0 {
                return true
            }
            false
        },
    }
}

/// Calculate the required delay for block making based on the block maker's
/// rank.
fn get_block_maker_delay(
    rank: u8,
) -> Option<Duration> {
    Some(NOTARIZATION_UNIT_DELAY * rank as u32)
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