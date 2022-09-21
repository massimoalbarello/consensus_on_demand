pub mod blockchain {

    use chrono::prelude::Utc;
    use serde::{Deserialize, Serialize};
    use sha2::{Digest, Sha256};

    pub type InputPayloads = Vec<String>;

    const DIFFICULTY_PREFIX: &str = "00";

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct Block {
        pub id: u64,
        pub hash: String,
        pub previous_hash: String,
        pub timestamp: i64,
        pub data: String,
        pub nonce: u64,
    }

    impl Block {
        pub async fn new(id: u64, previous_hash: String, data: String) -> Self {
            let current_timestamp = Utc::now().timestamp();
            let (nonce, hash) = mine_block(id, current_timestamp, &previous_hash, &data).await;
            Self {
                id,
                hash,
                timestamp: current_timestamp,
                previous_hash,
                data,
                nonce,
            }
        }
    }

    async fn mine_block(id: u64, timestamp: i64, previous_hash: &str, data: &str) -> (u64, String) {
        println!("Mining block...");
        let mut nonce = 0;

        loop {
            let hash = calculate_hash(id, timestamp, previous_hash, data, nonce);
            let binary_hash = hash_to_binary_representation(&hash);
            if binary_hash.starts_with(DIFFICULTY_PREFIX) {
                println!(
                    "Mined block with nonce: {} and hash: {}",
                    nonce,
                    hex::encode(&hash)
                );
                return (nonce, hex::encode(hash));
            }
            nonce += 1;
        }
    }

    fn calculate_hash(
        id: u64,
        timestamp: i64,
        previous_hash: &str,
        data: &str,
        nonce: u64,
    ) -> Vec<u8> {
        let data = serde_json::json!({
            "id": id,
            "previous_hash": previous_hash,
            "data": data,
            "timestamp": timestamp,
            "nonce": nonce
        });
        let mut hasher = Sha256::new();
        hasher.update(data.to_string().as_bytes());
        hasher.finalize().as_slice().to_owned()
    }

    fn hash_to_binary_representation(hash: &[u8]) -> String {
        let mut res: String = String::default();
        for c in hash {
            res.push_str(&format!("{:b}", c));
        }
        res
    }

    pub struct Blockchain {
        pub blocks: Vec<Block>,
    }

    impl Blockchain {
        pub async fn new() -> Self {
            let genesis_id: u64 = 0;
            let genesis_timestamp: i64 = 0;
            let genesis_previous_hash = String::from("Genesis block has no previous hash");
            let genesis_data = String::from("This is the genesis block!");
            let (genesis_nonce, genesis_hash) = mine_block(
                genesis_id,
                genesis_timestamp,
                &genesis_previous_hash,
                &genesis_data,
            )
            .await;
            let genesis_block = Block {
                id: genesis_id,
                hash: genesis_hash,
                timestamp: genesis_timestamp,
                previous_hash: genesis_previous_hash,
                data: genesis_data,
                nonce: genesis_nonce,
            };
            println!("Local blockchain initialized with genesis block");
            Self {
                blocks: vec![genesis_block],
            }
        }

        pub fn try_add_block(&mut self, block: Block) {
            let latest_block = self.blocks.last().expect("there is at least one block");
            if self.is_block_valid(&block, latest_block) {
                println!("Received block added to local blockchain");
                self.blocks.push(block);
            } else {
                println!("Could not add block - invalid");
            }
        }

        fn is_block_valid(&self, block: &Block, previous_block: &Block) -> bool {
            if block.previous_hash != previous_block.hash {
                println!("Block with id: {} has wrong previous hash", block.id);
                return false;
            } else if !hash_to_binary_representation(
                &hex::decode(&block.hash).expect("can decode from hex"),
            )
            .starts_with(DIFFICULTY_PREFIX)
            {
                println!("Block with id: {} has invalid difficulty", block.id);
                return false;
            } else if block.id != previous_block.id + 1 {
                println!(
                    "Block with id: {} is not the next block after the latest: {}",
                    block.id, previous_block.id
                );
                return false;
            } else if hex::encode(calculate_hash(
                block.id,
                block.timestamp,
                &block.previous_hash,
                &block.data,
                block.nonce,
            )) != block.hash
            {
                println!("Block with id: {} has invalid hash", block.id);
                return false;
            }
            true
        }
    }
}
