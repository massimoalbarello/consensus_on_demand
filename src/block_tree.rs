use std::rc::Rc;

pub struct Block {
    parent_ref: Option<Rc<Block>>,
    payload: String,
    height: u32,
}

impl Block {
    fn new(parent_ref: Option<Rc<Block>>, payload: String, height: u32) -> Self {
        Self {
            parent_ref,
            payload,
            height,
        }
    }
}

struct BlockTree {
    previous_tips_refs: Vec<Rc<Block>>,
    tips_refs: Vec<Rc<Block>>,
    current_height: u32,
}

impl BlockTree {
    fn new() -> Self {
        Self {
            previous_tips_refs: vec![Rc::new(Block::new(None, String::from("Genesis"), 0))],
            tips_refs: vec![],
            current_height: 0,
        }
    }

    fn create_child_at_height(
        &mut self,
        child_height: u32,
        index_in_tips_refs: usize,
        payload: String,
    ) {
        if child_height == self.current_height + 1 {
            match self.previous_tips_refs.get(index_in_tips_refs) {
                Some(parent_ref) => {
                    self.tips_refs.push(Rc::new(Block::new(
                        Some(Rc::clone(parent_ref)),
                        payload,
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
                    self.tips_refs = vec![Rc::new(Block::new(
                        Some(Rc::clone(parent_ref)),
                        payload,
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

    fn display_chain_from_tip(&self, index_in_tips_refs: usize) {
        match self.tips_refs.get(index_in_tips_refs) {
            Some(mut block) => {
                loop {
                    println!("{} at height {}", block.payload, block.height);
                    block = match block.parent_ref.as_ref() {
                        Some(parent) => parent,
                        None => break,
                    }
                }
                println!("");
            },
            None => println!("Invalid tip index"),
        }
    }
}

fn main() {
    let mut block_tree = BlockTree::new();

    block_tree.create_child_at_height(1, 0, String::from("Block 1_a"));
    block_tree.create_child_at_height(1, 0, String::from("Block 1_b"));
    block_tree.create_child_at_height(1, 0, String::from("Block 1_c"));

    block_tree.create_child_at_height(2, 0, String::from("Block 2_a_a"));
    block_tree.create_child_at_height(2, 0, String::from("Block 2_a_b"));

    block_tree.create_child_at_height(2, 1, String::from("Block 2_b_a"));
    block_tree.create_child_at_height(2, 1, String::from("Block 2_b_b"));

    block_tree.display_chain_from_tip(0);
    block_tree.display_chain_from_tip(1);
    block_tree.display_chain_from_tip(2);
    block_tree.display_chain_from_tip(3);
}
