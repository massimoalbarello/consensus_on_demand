pub mod blockchain {

    use std::{collections::BTreeMap, sync::{RwLock, Arc}};

    use chrono::prelude::Utc;
    use serde::{Deserialize, Serialize};
    use sha2::{Digest, Sha256};

    use crate::artifact_manager::processor::ProcessingResult;

    pub const N: usize = 4;

    pub type InputPayloads = Vec<String>;

    #[derive(Serialize, Deserialize, Debug)]
    pub enum Artifact {
        NotarizationShare(NotarizationShare),
        Block(Block),
        KeepAliveMessage,
    }

    // Unvalidated artifact
    #[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
    pub struct UnvalidatedArtifact<T> {
        pub message: T,
        pub peer_id: u8,
    }

    impl<T> UnvalidatedArtifact<T> {
        pub fn new(artifact: T) -> Self {
            Self {
                message: artifact,
                peer_id: 0,
            }
        }
    }

    #[derive(Serialize, Deserialize, Clone, Debug)]
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

    pub struct ConsensusProcessor {
        consensus_pool: Arc<RwLock<ConsensusPoolImpl>>,
        client: Box<ConsensusImpl>,
    }

    impl ConsensusProcessor {
        pub fn new() -> Self {
            Self {
                consensus_pool: Arc::new(RwLock::new(ConsensusPoolImpl::new())),
                client: Box::new(ConsensusImpl::new()),
            }
        }
    }

    impl ConsensusProcessor {
        pub fn process_changes(&self, artifacts: Vec<UnvalidatedArtifact<Artifact>>) -> ProcessingResult {
            if artifacts.len() != 0 {
                println!("{:?}", artifacts);
            }
            ProcessingResult::StateChanged
        }
    }

    pub struct InMemoryPoolSection {
        artifacts: BTreeMap<String, Artifact>,
    }

    impl InMemoryPoolSection {
        pub fn new() -> InMemoryPoolSection {
            InMemoryPoolSection {
                artifacts: BTreeMap::new(),
            }
        }
    }

    pub struct ConsensusPoolImpl {
        validated: Box<InMemoryPoolSection>,
        unvalidated: Box<InMemoryPoolSection>,
    }

    impl ConsensusPoolImpl {
        pub fn new() -> Self {
            Self {
                validated: Box::new(InMemoryPoolSection::new()),
                unvalidated: Box::new(InMemoryPoolSection::new()),
            }
        }
    }

    pub struct ConsensusImpl {
        notary: String,
    }

    impl ConsensusImpl {
        pub fn new() -> Self {
            Self {
                notary: String::from("notary"),
            }
        }
    }
}
