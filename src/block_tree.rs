use std::{cell::RefCell, rc::Rc};

use crate::consensus_layer::blockchain::{Block, N};

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

pub struct BlockTree {
    previous_round_tips_refs: Vec<Rc<RefCell<BlockWithRef>>>,
    current_round_tips_refs: Vec<Rc<RefCell<BlockWithRef>>>,
}

impl BlockTree {
    pub fn new(genesis: Block) -> Self {
        Self {
            previous_round_tips_refs: vec![Rc::new(RefCell::new(BlockWithRef::new(None, genesis)))],
            current_round_tips_refs: vec![],
        }
    }

    pub fn get_parent_hash(
        &mut self,
        child_height: u64,
    ) -> Option<String> {
        for parent_ref in self.previous_round_tips_refs.iter() {
            if parent_ref.borrow().block.from_rank == 0 {
                return Some(parent_ref.borrow().block.hash.to_owned());
            }
        }
        None
    }

    pub fn append_child_to_previous_leader(&mut self, block: Block) {
        // local peer receives only blocks at height corresponding to the current round
        // these have to be appended to blocks of the previous round (referenced by previous_round_tips_refs)
        for parent_ref in self.previous_round_tips_refs.iter() {
            if parent_ref.borrow().block.from_rank == 0 {
                println!(
                    "\nBlock at height: {} appended to previous leader with hash: {}",
                    block.height, parent_ref.borrow().block.hash
                );
                self.current_round_tips_refs
                    .push(Rc::new(RefCell::new(BlockWithRef::new(
                        Some(Rc::clone(parent_ref)),
                        block.clone(),
                    ))));
            }
        }
    }

    pub fn update_block_with_ref(
        &mut self,
        from_node_number: u8,
        block_hash: &str,
        block_height: u64,
        current_round: u64,
    ) -> bool {
        if block_height == current_round {
            for (index_of_tip_ref, tip_ref) in self.current_round_tips_refs.iter().enumerate() {
                if tip_ref.borrow().block.hash.eq(block_hash) {
                    return self.update_recvd_notarization_shares(
                        Rc::clone(&self.current_round_tips_refs[index_of_tip_ref]),
                        from_node_number,
                    );
                }
            }
            return false;
        } else if block_height == current_round - 1 {
            println!("Received share for block at height: {}", block_height);
            for (index_of_tip_ref, tip_ref) in self.previous_round_tips_refs.iter().enumerate() {
                if tip_ref.borrow().block.hash.eq(block_hash) {
                    return self.update_recvd_notarization_shares(
                        Rc::clone(&self.previous_round_tips_refs[index_of_tip_ref]),
                        from_node_number,
                    );
                }
            }
            return false;
        } else {
            println!(
                "Ignoring received notarization share for block with hash: {} at height: {}",
                block_hash, block_height
            );
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
            println!(
                "\n{} --->",
                block.hash
            );
            let mut parent_ref = tip_ref.borrow().parent_ref.clone();
            loop {
                parent_ref = match parent_ref {
                    Some(parent) => {
                        let block = parent.borrow().block.clone();
                        println!(
                            "{} --->",
                            block.hash
                        );
                        parent.borrow().parent_ref.clone()
                    },
                    None => {
                        println!("()");
                        break;
                    }
                }
            }
        }
    }
}
