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
                println!("Created block in block tree received from peer with node number: {} with notarization shares from: {:?}", block.from_node_number, recvd_notarization_shares);
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
        index_in_tips_refs: usize,
    ) -> Option<String> {
        match self.previous_round_tips_refs.get(index_in_tips_refs) {
            Some(parent_ref) => Some(parent_ref.borrow().block.hash.to_owned()),
            None => {
                println!(
                    "No parent at height {} and width {}",
                    child_height - 1,
                    index_in_tips_refs
                );
                None
            }
        }
    }

    pub fn create_child_at_index(&mut self, index_in_tips_refs: usize, block: Block) {
        // local peer receives only blocks at height corresponding to the current round
        // these have to be appended to blocks of the previous round (referenced by previous_round_tips_refs)
        match self.previous_round_tips_refs.get(index_in_tips_refs) {
            Some(parent_ref) => {
                let height = block.height;
                let parent_hash = block.parent_hash.clone();
                self.current_round_tips_refs
                    .push(Rc::new(RefCell::new(BlockWithRef::new(
                        Some(Rc::clone(parent_ref)),
                        block,
                    ))));
                println!(
                    "\nBlock at height: {} appended to parent with hash: {}",
                    height, parent_hash
                );
            }
            None => println!(
                "No parent at height {} and width {}",
                block.height - 1,
                index_in_tips_refs
            ),
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

    // pub fn display_chain_from_tip(&self, index_in_tips_refs: usize) {
    //     match self.current_round_tips_refs.get(index_in_tips_refs) {
    //         Some(mut block_with_ref) => {
    //             loop {
    //                 println!(
    //                     "Block with payload: '{}' at height: {}",
    //                     block_with_ref.borrow().block.payload, block_with_ref.borrow().block.height
    //                 );
    //                 block_with_ref = match block_with_ref.parent_ref.as_ref() {
    //                     Some(parent) => parent,
    //                     None => break,
    //                 }
    //             }
    //             println!("");
    //         }
    //         None => println!("Invalid tip index"),
    //     }
    // }
}
