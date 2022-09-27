use std::{cell::RefCell, rc::Rc};

use crate::consensus_layer::blockchain::{Block, N, NotarizationShare, Artifact};

pub struct BlockWithRef {
    parent_ref: Option<Rc<RefCell<BlockWithRef>>>,
    block: Block,
    recvd_notarization_shares: Vec<bool>,
    is_notarized: bool,
}

impl BlockWithRef {
    fn new(parent_ref: Option<Rc<RefCell<BlockWithRef>>>, block: Block) -> Self {
        Self {
            parent_ref,
            recvd_notarization_shares: {
                let mut recvd_notarization_shares = vec![false; N];
                recvd_notarization_shares[(block.from_node_number - 1) as usize] = true; // remote peer broadcasts its notarization share right after the block
                recvd_notarization_shares
            },
            block,
            is_notarized: false,
        }
    }
}

struct StoreArtifacts {
    // in current round local peer can receive both shares and blocks for the next round, store it for next round
    next_round_shares: Vec<NotarizationShare>,
    next_round_blocks: Vec<Block>,
    // in current_round local peer can receive shares for a block of the current round before it receives the block, store it until block arrives
    current_round_shares: Vec<NotarizationShare>,
}

impl StoreArtifacts {
    fn new() -> Self {
        Self {
            next_round_shares: vec![],
            next_round_blocks: vec![],
            current_round_shares: vec![],
        }
    }

    fn push(&mut self, artifact: Artifact, current_round: u64) {
        match artifact {
            Artifact::NotarizationShare(share) => {
                if share.block_height == current_round {
                    self.current_round_shares.push(share);
                }
                else {
                    self.next_round_shares.push(share);
                }
            },
            Artifact::Block(block) => {
                self.next_round_blocks.push(block);
            },
            _ => (),
        }
        self.display_artifacts_store();
    }

    fn display_artifacts_store(&self) {
        println!("------------- Artifacts pool contains: 
        - for current round: {} shares 
        -for next round: {} shares and {} blocks", 
        self.current_round_shares.len(), self.next_round_shares.len(), self.next_round_blocks.len());
    }
}

pub struct BlockTree {
    previous_round_tips_refs: Vec<Rc<RefCell<BlockWithRef>>>,
    current_round_tips_refs: Vec<Rc<RefCell<BlockWithRef>>>,
    artifacts_store: StoreArtifacts,
}

impl BlockTree {
    pub fn new(genesis: Block) -> Self {
        Self {
            previous_round_tips_refs: vec![Rc::new(RefCell::new(BlockWithRef::new(None, genesis)))],
            current_round_tips_refs: vec![],
            artifacts_store: StoreArtifacts::new(),
        }
    }

    pub fn get_previous_leader_hash(&mut self) -> Option<String> {
        for parent_ref in self.previous_round_tips_refs.iter() {
            if parent_ref.borrow().block.from_rank == 0 {
                return Some(parent_ref.borrow().block.hash.to_owned());
            }
        }
        None
    }

    pub fn append_child_to_previous_leader(&mut self, block: Block, current_round: u64) {
        if block.height == current_round {
            // if local peer receives a block at height corresponding to the current round append it to the block broadcasted by the leader of the previous round
            for parent_ref in self.previous_round_tips_refs.iter() {
                if parent_ref.borrow().block.from_rank == 0 {
                    println!(
                        "\nBlock with hash: {} at height: {} appended to previous leader with hash: {}",
                        block.hash,
                        block.height,
                        parent_ref.borrow().block.hash
                    );
                    self.current_round_tips_refs
                        .push(Rc::new(RefCell::new(BlockWithRef::new(
                            Some(Rc::clone(parent_ref)),
                            block.clone(),
                        ))));
                }
            }
        }
        else if block.height > current_round {
            println!("Received block for next round");
            self.artifacts_store.push(Artifact::Block(block), current_round);
        }
        else {
            println!("============ Received block for previous round. Block for height {}", block.height);
        }
    }

    pub fn update_block_with_ref(
        &mut self,
        share: NotarizationShare,
        current_round: u64,
    ) -> bool {
        if share.block_height == current_round {
            for (index_of_tip_ref, tip_ref) in self.current_round_tips_refs.iter().enumerate() {
                if tip_ref.borrow().block.hash.eq(&share.block_hash) {
                    return self.update_recvd_notarization_shares(
                        Rc::clone(&self.current_round_tips_refs[index_of_tip_ref]),
                        share.from_node_number,
                    );
                }
            }
            return false;
        } else if share.block_height == current_round - 1 {
            println!("Received share for block at height: {}", share.block_height);
            for (index_of_tip_ref, tip_ref) in self.previous_round_tips_refs.iter().enumerate() {
                if tip_ref.borrow().block.hash.eq(&share.block_hash) {
                    return self.update_recvd_notarization_shares(
                        Rc::clone(&self.previous_round_tips_refs[index_of_tip_ref]),
                        share.from_node_number,
                    );
                }
            }
            return false;
        } else {
            if share.block_height < current_round {
                println!(
                    "??????????????????? Ignoring notarization share for block with hash: {} at height: {}",
                    share.block_hash, share.block_height
                );
            }
            else {
                // store share so that it can be added to respective block as soon as it arrives or once next round starts
                println!(
                    "!!!!!!!!!!!!!!!!!!! Received notarization share for block with hash: {} at height: {}",
                    share.block_hash, share.block_height
                );
                self.artifacts_store.push(Artifact::NotarizationShare(share), current_round);
            }
        }
        false
    }

    fn update_recvd_notarization_shares(
        &mut self,
        block_with_ref: Rc<RefCell<BlockWithRef>>,
        from_node_number: u8,
    ) -> bool {
        let block = block_with_ref.borrow().block.clone();
        block_with_ref.borrow_mut().recvd_notarization_shares[(from_node_number - 1) as usize] =
            true;
        println!(
            "Block with hash {} has received notarization shares from: {:?}",
            block.hash,
            block_with_ref.borrow().recvd_notarization_shares
        );
        // check if round has to be updated only if the block has not been already notarized (as the round would have already been updated then)
        if !block_with_ref.borrow().is_notarized {
            // if exactly N-1 notarization shares are received, check if there is another block at the same height which has already been notarized
            // if not, return true (trigger round update in network layer)
            // otherwise, return false (remain in current round)
            // upon receiving Nth share for a block, do not update the round as it has already been updated
            if block_with_ref
                .borrow()
                .recvd_notarization_shares
                .iter()
                .filter(|&is_received| *is_received == true)
                .count()
                == N - 1
            {
                block_with_ref.borrow_mut().is_notarized = true;
                // if it is the first block being notarized at this height, trigger round update
                if self.count_blocks_notarized_at_same_height() == 1 {
                    println!("Found first notarized block at height: {}", block.height);
                    self.display_block_tree();
                    return true;
                }
            }
        }
        false
    }

    pub fn check_if_artifacts_already_received(&mut self, current_round: u64) {
        if self.artifacts_store.next_round_blocks.len() > 0 {
            println!("++++++++++++++ Block for round: {} has already been received", current_round);
            // add blocks for next round (received the round before) to block tree
            // for now there can be only one such block as in each round only one peer broadcast a block for that height
            let block_to_be_appended = self.artifacts_store.next_round_blocks[0].clone();
            self.append_child_to_previous_leader(block_to_be_appended, current_round);
            self.artifacts_store.next_round_blocks = vec![];
        }
    }

    fn count_blocks_notarized_at_same_height(&self) -> usize {
        self.current_round_tips_refs
            .iter()
            .filter(|&block_with_ref| block_with_ref.borrow().is_notarized == true)
            .count()
    }

    pub fn update_tips_refs(&mut self) {
        self.previous_round_tips_refs = self.current_round_tips_refs.to_owned();
        self.current_round_tips_refs = vec![];
    }

    pub fn display_block_tree(&self) {
        for tip_ref in self.current_round_tips_refs.iter() {
            let block = tip_ref.borrow().block.clone();
            println!("\n{} --->", block.hash);
            let mut parent_ref = tip_ref.borrow().parent_ref.clone();
            loop {
                parent_ref = match parent_ref {
                    Some(parent) => {
                        let block = parent.borrow().block.clone();
                        println!("{} --->", block.hash);
                        parent.borrow().parent_ref.clone()
                    }
                    None => {
                        println!("()");
                        break;
                    }
                }
            }
        }
    }
}
