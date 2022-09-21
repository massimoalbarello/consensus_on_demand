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

fn create_child(parent: Rc<Block>, payload: String) -> Rc<Block> {
    Rc::new(Block::new(Some(Rc::clone(&parent)), payload))
}

fn display_tree_from_tip(tip_ref: Rc<Block>) {
    let mut block = tip_ref.as_ref();
    loop {
        println!("{}", block.payload);
        block = match block.parent_ref.as_ref() {
            Some(block) => block,
            None => break,
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

    fn create_child(&mut self, payload: String) {
        self.tip_ref = Rc::new(Block::new(Some(Rc::clone(&self.tip_ref)), payload));
    }

    fn display_tree_from_tip(&self) {
        let mut block = &self.tip_ref;
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
    block_tree.create_child(String::from("Block 1"));
    block_tree.create_child(String::from("Block 2"));
    block_tree.create_child(String::from("Block 3"));
    block_tree.create_child(String::from("Block 4"));
    block_tree.display_tree_from_tip();
}
