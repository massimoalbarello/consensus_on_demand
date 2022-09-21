use std::rc::Rc;

pub struct Block {
    payload: String,
    parent_ref: Option<Rc<Block>>,
}

impl Block {
    fn new(parent_ref: Option<Rc<Block>>, payload: String) -> Self {
        Self {
            payload,
            parent_ref,
        }
    }
}

struct BlockTree {
    tip_ref: Rc<Block>,
}

impl BlockTree {
    fn new() -> Self {
        Self {
           tip_ref: Rc::new(Block::new(None, String::from("Genesis"))),
        }
    }

    fn create_child(&mut self, parent_ref: Rc<Block>, payload: String) {
        self.tip_ref = Rc::new(Block::new(Some(parent_ref), payload));
    }

    fn display_chain_from_tip(&self, mut block: &Rc<Block>) {
        loop {
            println!("{}", block.payload);
            block = match block.parent_ref.as_ref() {
                Some(parent) => parent,
                None => break,
            }
        }
    }
}

fn main() {
    let mut block_tree = BlockTree::new();
    let genesis_ref_a = Rc::clone(&block_tree.tip_ref);
    let genesis_ref_b = Rc::clone(&block_tree.tip_ref);

    block_tree.create_child(genesis_ref_a, String::from("Block 1a"));
    let block_1a_ref = Rc::clone(&block_tree.tip_ref);

    block_tree.create_child(genesis_ref_b, String::from("Block 1b"));
    let block_1b_ref = Rc::clone(&block_tree.tip_ref);

    block_tree.create_child(block_1a_ref, String::from("Block 2a"));
    let block_2a_ref = Rc::clone(&block_tree.tip_ref);

    block_tree.create_child(block_1b_ref, String::from("Block 2b"));
    let block_2b_ref = Rc::clone(&block_tree.tip_ref);

    block_tree.display_chain_from_tip(&block_2a_ref);
    block_tree.display_chain_from_tip(&block_2b_ref);

}
