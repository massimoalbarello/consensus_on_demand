pub mod blockchain {

    use chrono::prelude::Utc;
    use serde::{Deserialize, Serialize};
    use sha2::{Digest, Sha256};

    use crate::block_tree::BlockTree;

    pub type InputPayloads = Vec<String>;

    pub const N: usize = 4;

    #[derive(Serialize, Deserialize)]
    pub enum Artifact {
        NotarizationShare(NotarizationShare),
        Block(Block),
        KeepAliveMessage,
    }

    #[derive(Serialize, Deserialize, Clone)]
    pub struct NotarizationShare {
        pub from_node_number: u8,
        pub block_height: u64,
        pub block_hash: String,
    }

    impl NotarizationShare {
        pub fn new(from_node_number: u8, height: u64, hash: String) -> Self {
            Self {
                from_node_number,
                block_height: height,
                block_hash: hash,
            }
        }
    }

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct Block {
        pub height: u64,
        pub from_rank: u64,
        pub from_node_number: u8,
        pub hash: String,
        pub parent_hash: String,
        pub timestamp: i64,
        pub payload: String,
    }

    impl Block {
        pub fn new(
            height: u64,
            from_rank: u64,
            from_node_number: u8,
            parent_hash: String,
            payload: String,
        ) -> Self {
            let current_timestamp = Utc::now().timestamp();
            let hash = calculate_hash(height, current_timestamp, &parent_hash, &payload);
            println!("Created block with hash {}", &hash);
            Self {
                height,
                from_rank,
                from_node_number,
                hash,
                timestamp: current_timestamp,
                parent_hash,
                payload,
            }
        }
    }

    fn calculate_hash(height: u64, timestamp: i64, parent_hash: &str, payload: &str) -> String {
        let payload = serde_json::json!({
            "height": height,
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
            let genesis_height: u64 = 0;
            let genesis_timestamp: i64 = 0;
            let genesis_parent_hash = String::from("Genesis block has no previous hash");
            let genesis_payload = String::from("This is the genesis block!");
            let genesis_hash = calculate_hash(
                genesis_height,
                genesis_timestamp,
                &genesis_parent_hash,
                &genesis_payload,
            );
            let genesis_block = Block {
                height: genesis_height,
                from_rank: 0,        // irrelevant as genesis block is not broadcasted
                from_node_number: 1, // irrelevant as genesis block does not receive notarization shares
                hash: genesis_hash,
                timestamp: genesis_timestamp,
                parent_hash: genesis_parent_hash,
                payload: genesis_payload,
            };
            println!(
                "Local blockchain initialized with genesis block with hash {}",
                &genesis_block.hash
            );
            Self {
                block_tree: BlockTree::new(genesis_block),
                finalized_chain_index: 0,
            }
        }
    }
}
