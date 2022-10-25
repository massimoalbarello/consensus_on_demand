use serde::{Serialize, Deserialize};

use crate::{consensus_layer::{
    pool_reader::PoolReader,
    artifacts::{ConsensusMessage, N}
}, crypto::{Signed, Hashed}};

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
    rank: u8,
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
}

impl BlockMaker {
    pub fn new(node_id: u8) -> Self {
        Self {
            node_id,
        }
    }

    pub fn on_state_change(&self, pool: &PoolReader<'_>) -> Option<ConsensusMessage> {
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
                        my_node_id
                    )
                {
                    println!("\n########## Block maker ##########");
                    let block_proposal = self.propose_block(pool, rank, parent).map(|proposal| {
                        ConsensusMessage::BlockProposal(proposal)
                    });
                    println!("Block proposal: {:?}", block_proposal);
                    block_proposal
                }
                else {
                    None
                }
            }
        }
    }

    fn is_better_block_proposal_available(
        &self,
        pool: &PoolReader<'_>,
        height: u64,
        rank: u8,
    ) -> bool {
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
    let parent = pool
        .get_notarized_blocks(notarized_height)
        .min_by(|block1, block2| block1.rank.cmp(&block2.rank));
    match parent {
        Some(parent) => {
            println!("Parent block: {:?}", parent);
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
    println!("Local rank for height {} is: {}", height, rank);
    rank
}

// Return true if this node has already made a proposal at the given height.
fn already_proposed(pool: &PoolReader<'_>, h: u64, this_node: u8) -> bool {
    match pool.pool().validated().block_proposal().max_height() {
        Some(last_proposed_block_height) => last_proposed_block_height == h,
        None => false
    }
}

// Return true if the time since round start is greater than the required block
// maker delay for the given rank.
pub fn is_time_to_make_block(
    pool: &PoolReader<'_>,
    height: u64,
    rank: u8,
    node_id: u8
) -> bool {
    rank == 0
}