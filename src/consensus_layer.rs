pub mod blockchain {

    use chrono::prelude::Utc;
    use serde::{Deserialize, Serialize};
    use sha2::{Digest, Sha256};

    use crate::block_tree::{BlockTree, BlockWithRef};

    pub type InputPayloads = Vec<String>;

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct Block {
        pub id: u64,
        pub hash: String,
        pub parent_hash: String,
        pub timestamp: i64,
        pub payload: String,
    }

    impl Block {
        pub fn new(id: u64, parent_hash: String, payload: String) -> Self {
            let current_timestamp = Utc::now().timestamp();
            let hash = calculate_hash(id, current_timestamp, &parent_hash, &payload);
            println!("Created block with hash {}", &hash);
            Self {
                id,
                hash,
                timestamp: current_timestamp,
                parent_hash,
                payload,
            }
        }
    }
    
    fn calculate_hash(
        id: u64,
        timestamp: i64,
        parent_hash: &str,
        payload: &str,
    ) -> String {
        let payload = serde_json::json!({
            "id": id,
            "parent_hash": parent_hash,
            "payload": payload,
            "timestamp": timestamp,
        });
        let mut hasher = Sha256::new();
        hasher.update(payload.to_string().as_bytes());
        hex::encode(hasher.finalize().as_slice().to_owned())
    }

    pub struct Blockchain {
        pub block_tree: BlockTree,
        pub finalized_chain_index: usize,
    }

    impl Blockchain {
        pub fn new() -> Self {
            let genesis_id: u64 = 0;
            let genesis_timestamp: i64 = 0;
            let genesis_parent_hash = String::from("Genesis block has no previous hash");
            let genesis_payload = String::from("This is the genesis block!");
            let genesis_hash = calculate_hash(genesis_id, genesis_timestamp, &genesis_parent_hash, &genesis_payload);
            let genesis_block = Block {
                id: genesis_id,
                hash: genesis_hash,
                timestamp: genesis_timestamp,
                parent_hash: genesis_parent_hash,
                payload: genesis_payload,
            };
            println!("Local blockchain initialized with genesis block with hash {}", &genesis_block.hash);
            Self {
                block_tree: BlockTree::new(genesis_block),
                finalized_chain_index: 0,
            }
        }

        pub fn try_add_block(&mut self, block: Block) {
            // let latest_block = self.blocks.last().expect("there is at least one block");
            // if self.is_block_valid(&block, latest_block) {
            //     println!("Received block added to local blockchain");
            //     self.blocks.push(block);
            // } else {
            //     println!("Could not add block: invalid");
            // }
        }

        fn is_block_valid(&self, block: &Block, previous_block: &Block) -> bool {
            if block.parent_hash != previous_block.hash {
                println!("Block with id: {} has wrong previous hash", block.id);
                return false;
            } else if block.id != previous_block.id + 1 {
                println!(
                    "Block with id: {} is not the next block after the latest: {}",
                    block.id, previous_block.id
                );
                return false;
            } else if calculate_hash(
                block.id,
                block.timestamp,
                &block.parent_hash,
                &block.payload,
            ) != block.hash
            {
                println!("Block with id: {} has invalid hash", block.id);
                return false;
            }
            true
        }
    }
}
