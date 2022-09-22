use std::rc::Rc;

use crate::consensus_layer::blockchain::Block;

pub struct BlockWithRef {
    parent_ref: Option<Rc<BlockWithRef>>,
    block: Block,
    height: u32,
}

impl BlockWithRef {
    fn new(parent_ref: Option<Rc<BlockWithRef>>, block: Block, height: u32) -> Self {
        Self {
            parent_ref,
            block,
            height,
        }
    }
}

pub struct BlockTree {
    previous_tips_refs: Vec<Rc<BlockWithRef>>,
    tips_refs: Vec<Rc<BlockWithRef>>,
    current_height: u32,
}

impl BlockTree {
    pub fn new(genesis: Block) -> Self {
        Self {
            previous_tips_refs: vec![Rc::new(BlockWithRef::new(None, genesis, 0))],
            tips_refs: vec![],
            current_height: 0,
        }
    }

    pub fn get_current_height(&self) -> u32 {
        self.current_height
    }

    pub fn get_parent_hash(
        &mut self,
        child_height: u32,
        index_in_tips_refs: usize,
    ) -> Option<String> {
        if child_height == self.current_height + 1 {
            match self.previous_tips_refs.get(index_in_tips_refs) {
                Some(parent_ref) => Some(parent_ref.block.hash.to_owned()),
                None => {
                    println!(
                        "No parent at height {} and width {}",
                        child_height - 1,
                        index_in_tips_refs
                    );
                    None
                }
            }
        } else if child_height == self.current_height + 2 {
            match self.tips_refs.get(index_in_tips_refs) {
                Some(parent_ref) => Some(parent_ref.block.hash.to_owned()),
                None => {
                    println!(
                        "No parent at height {} and width {}",
                        child_height, index_in_tips_refs
                    );
                    None
                }
            }
        } else {
            println!("Invalid child height");
            None
        }
    }

    pub fn create_child_at_height(
        &mut self,
        child_height: u32,
        index_in_tips_refs: usize,
        block: Block,
    ) {
        if child_height == self.current_height + 1 {
            match self.previous_tips_refs.get(index_in_tips_refs) {
                Some(parent_ref) => {
                    self.tips_refs.push(Rc::new(BlockWithRef::new(
                        Some(Rc::clone(parent_ref)),
                        block,
                        child_height,
                    )));
                }
                None => println!(
                    "No parent at height {} and width {}",
                    child_height - 1,
                    index_in_tips_refs
                ),
            }
        } else if child_height == self.current_height + 2 {
            match self.tips_refs.get(index_in_tips_refs) {
                Some(parent_ref) => {
                    self.previous_tips_refs = self.tips_refs.to_owned();
                    self.tips_refs = vec![Rc::new(BlockWithRef::new(
                        Some(Rc::clone(parent_ref)),
                        block,
                        child_height,
                    ))];
                    self.current_height += 1;
                }
                None => println!(
                    "No parent at height {} and tip index {}",
                    child_height, index_in_tips_refs
                ),
            }
        } else {
            println!("Invalid child height");
        }
    }

    pub fn display_chain_from_tip(&self, index_in_tips_refs: usize) {
        match self.tips_refs.get(index_in_tips_refs) {
            Some(mut block_with_ref) => {
                loop {
                    println!(
                        "Block with payload: '{}' at height: {}",
                        block_with_ref.block.payload, block_with_ref.height
                    );
                    block_with_ref = match block_with_ref.parent_ref.as_ref() {
                        Some(parent) => parent,
                        None => break,
                    }
                }
                println!("");
            }
            None => println!("Invalid tip index"),
        }
    }
}

// fn main() {
//     let mut block_tree = BlockTree::new();

//     block_tree.create_child_at_height(1, 0, String::from("Block 1_a"));
//     block_tree.create_child_at_height(1, 0, String::from("Block 1_b"));
//     block_tree.create_child_at_height(1, 0, String::from("Block 1_c"));

//     block_tree.display_chain_from_tip(0);
//     block_tree.display_chain_from_tip(1);
//     block_tree.display_chain_from_tip(2);

//     block_tree.create_child_at_height(2, 0, String::from("Block 2_a_a"));
//     block_tree.create_child_at_height(2, 0, String::from("Block 2_a_b"));

//     block_tree.create_child_at_height(2, 1, String::from("Block 2_b_a"));
//     block_tree.create_child_at_height(2, 1, String::from("Block 2_b_b"));

//     block_tree.display_chain_from_tip(0);
//     block_tree.display_chain_from_tip(1);
//     block_tree.display_chain_from_tip(2);
//     block_tree.display_chain_from_tip(3);
// }
