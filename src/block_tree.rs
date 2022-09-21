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
}

fn main() {
    let block_tree = BlockTree::new();
    let block1 = create_child(block_tree.tip_ref, String::from("Block 1"));
    let block2 = create_child(block1, String::from("Block 2"));
    let block3 = create_child(block2, String::from("Block 3"));

    let block_tree_tip_ref = Rc::clone(&block3);
    display_tree_from_tip(block_tree_tip_ref);
}
