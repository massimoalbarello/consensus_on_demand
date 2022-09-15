pub mod blockchain {

    use async_std::{fs::File, prelude::*};
    use chrono::prelude::Utc;
    use libp2p::PeerId;
    use serde::{Deserialize, Serialize};
    use sha2::{Digest, Sha256};

    type InputPayloads = Vec<String>;

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
        pub fn new(id: u64, previous_hash: String, data: String) -> Self {
            let current_timestamp = Utc::now().timestamp();
            let (nonce, hash) = mine_block(id, current_timestamp, &previous_hash, &data);
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

    fn mine_block(id: u64, timestamp: i64, previous_hash: &str, data: &str) -> (u64, String) {
        println!("Mining block...");
        let mut nonce = 0;

        loop {
            let hash = calculate_hash(id, timestamp, previous_hash, data, nonce);
            let binary_hash = hash_to_binary_representation(&hash);
            if binary_hash.starts_with("00") {
                println!(
                    "mined! nonce: {}, hash: {}, binary hash: {}",
                    nonce,
                    hex::encode(&hash),
                    binary_hash
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

    pub async fn get_next_block(local_sn: usize) -> Option<Block> {
        match get_next_payload(local_sn).await {
            Some(payload) => Some(Block::new(local_sn as u64, String::from("aaa"), payload)),
            None => None,
        }
    }

    async fn get_next_payload(local_sn: usize) -> Option<String> {
        let input_payloads: InputPayloads = read_file("payloads_pool.txt").await;
        let next_payload = if local_sn < input_payloads.len() {
            Some(input_payloads[local_sn].clone())
        } else {
            None
        };
        next_payload
    }

    async fn read_file(path: &str) -> InputPayloads {
        let mut file = File::open(path).await.expect("txt file in path");
        let mut content = String::new();
        file.read_to_string(&mut content)
            .await
            .expect("read content as string");

        let mut input_payloads: InputPayloads = vec![];
        for line in content.lines() {
            input_payloads.push(String::from(line));
        }
        input_payloads
    }

    pub fn handle_incoming_block(content: &str, source: PeerId) {
        println!("{}: {}", source, content);
    }
}
