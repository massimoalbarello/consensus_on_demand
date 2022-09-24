use std::{cell::RefCell, rc::Rc};

use crate::consensus_layer::blockchain::{Block, N};

pub struct BlockWithRef {
    parent_ref: Option<Rc<RefCell<BlockWithRef>>>,
    block: Block,
    recvd_notarization_shares: Vec<bool>,
}

impl BlockWithRef {
    fn new(parent_ref: Option<Rc<RefCell<BlockWithRef>>>, block: Block) -> Self {
        Self {
            parent_ref,
            recvd_notarization_shares: {
                let mut recvd_notarization_shares = vec![false; N];
                recvd_notarization_shares[block.from_node_number as usize] = true; // remote peer broadcasts its notarization share right after the block
                recvd_notarization_shares
            },
            block,
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

    pub fn update_recvd_notarization_shares(
        &mut self,
        from_node_number: u8,
        block_hash: &str,
        block_height: u64,
        current_round: u64,
    ) {
        if block_height == current_round {
            for (index_of_tip_ref, tip_ref) in self.current_round_tips_refs.iter().enumerate() {
                if tip_ref.borrow().block.hash.eq(block_hash) {
                    self.previous_round_tips_refs[index_of_tip_ref]
                        .borrow_mut()
                        .recvd_notarization_shares[(from_node_number - 1) as usize] = true;
                    println!(
                        "Block with hash {} has received notarization shares from: {:?}",
                        block_hash,
                        self.previous_round_tips_refs[index_of_tip_ref]
                            .borrow()
                            .recvd_notarization_shares
                    );
                }
            }
        } else if block_height == current_round - 1 {
            for (index_of_tip_ref, tip_ref) in self.previous_round_tips_refs.iter().enumerate() {
                if tip_ref.borrow().block.hash.eq(block_hash) {
                    self.previous_round_tips_refs[index_of_tip_ref]
                        .borrow_mut()
                        .recvd_notarization_shares[(from_node_number - 1) as usize] = true;
                    println!(
                        "Block with hash {} has received notarization shares from: {:?}",
                        block_hash,
                        self.previous_round_tips_refs[index_of_tip_ref]
                            .borrow()
                            .recvd_notarization_shares
                    );
                }
            }
        } else {
            println!(
                "Ignoring received notatarization share for block with hash: {} at height: {}",
                block_hash, block_height
            );
        }
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
