pub mod blockchain {

    use serde::{Deserialize, Serialize};
    use async_std::{fs::File, prelude::*};
    use libp2p::PeerId;

    #[derive(Serialize, Deserialize)]
    struct InputBlocks {
        blocks: Vec<Block>
    }

    #[derive(Clone, Serialize, Deserialize)]
    pub struct Block {
        transactions: Vec<Transaction>,
    }

    #[derive(Clone, Serialize, Deserialize)]
    struct Transaction {
        sender: String,
        receiver: String,
        amount: u32,
    }

    pub async fn get_next_block(local_sn: usize) -> Option<Block> {
        let input_blocks = read_file("blocks_pool.txt").await;
        let next_block = if local_sn < input_blocks.blocks.len() {
            Some(input_blocks.blocks[local_sn].clone())
        }
        else {
            None
        };
        next_block
    }

    async fn read_file(path: &str) -> InputBlocks {
        let mut file = File::open(path).await.expect("txt file in path");
        let mut content = String::new();
        file.read_to_string(&mut content).await.expect("read content as string");

        let input_blocks: InputBlocks = serde_json::from_str(&content).expect("invalid json");
        input_blocks
    }

    pub fn handle_incoming_block(content: &str, source: PeerId) {
        println!("{}: {}", source, content);
    }
}