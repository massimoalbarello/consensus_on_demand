use chrono::prelude::Utc;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub type ChangeSet = Vec<ChangeAction>;

pub enum ChangeAction {
    AddToValidated(String),
    MoveToValidated(String),
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

pub fn calculate_hash(artifact: UnvalidatedArtifact<Artifact>) -> String {
    let payload = serde_json::json!(artifact);
    let mut hasher = Sha256::new();
    hasher.update(payload.to_string().as_bytes());
    hex::encode(hasher.finalize().as_slice().to_owned())
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Artifact {
    NotarizationShare(NotarizationShare),
    Block(Block),
    KeepAliveMessage,
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
        let hash = String::from("Block hash");
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
