pub mod blockchain {

    use std::{collections::BTreeMap, sync::{RwLock, Arc}};

    use chrono::prelude::Utc;
    use serde::{Deserialize, Serialize};
    use sha2::{Digest, Sha256};

    use crate::artifact_manager::processor::ProcessingResult;

    type ChangeSet = Vec<ChangeAction>;

    enum ChangeAction {
        AddToValidated(String),
        MoveToValidated(String),
    }

    #[derive(Serialize, Deserialize, Debug, Clone)]
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

    fn calculate_hash(artifact: UnvalidatedArtifact<Artifact>) -> String {
        let payload = serde_json::json!(artifact);
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
                let mut consensus_pool = self.consensus_pool.write().unwrap();
                for artifact in artifacts {
                    consensus_pool.insert(artifact)
                }
                return ProcessingResult::StateChanged;
            }
            ProcessingResult::StateUnchanged
        }
    }

    pub struct InMemoryPoolSection {
        artifacts: BTreeMap<String, UnvalidatedArtifact<Artifact>>,
    }

    impl InMemoryPoolSection {
        pub fn new() -> InMemoryPoolSection {
            InMemoryPoolSection {
                artifacts: BTreeMap::new(),
            }
        }

        fn mutate(&mut self, ops: PoolSectionOps<UnvalidatedArtifact<Artifact>>) {
            for op in ops.ops {
                match op {
                    PoolSectionOp::Insert(artifact) => self.insert(artifact),
                }
                println!("Inserted artifact in unvalidated section of consensus pool: {:?}", self.artifacts);
            }
        }

        fn insert(&mut self, artifact: UnvalidatedArtifact<Artifact>) {
            let hash = calculate_hash(artifact.clone());
            self.artifacts.entry(hash).or_insert(artifact);
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

        fn insert(&mut self, unvalidated_artifact: UnvalidatedArtifact<Artifact>) {
            let mut ops = PoolSectionOps::new();
            ops.insert(unvalidated_artifact);
            self.apply_changes_unvalidated(ops);
        }

        fn apply_changes_unvalidated(&mut self, ops: PoolSectionOps<UnvalidatedArtifact<Artifact>>) {
            if !ops.ops.is_empty() {
                self.unvalidated.mutate(ops);
            }
        }

    }

    /// Consensus message identifier carries both a message hash and a height,
    /// which is used by the consensus pool to help lookup.
    #[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub struct ConsensusMessageId {
        pub hash: String,
        pub height: u64,
    }

    #[derive(Debug, Clone)]
    pub enum PoolSectionOp<T> {
        Insert(T),
    }

    #[derive(Clone, Debug, Default)]
    pub struct PoolSectionOps<T> {
        pub ops: Vec<PoolSectionOp<T>>,
    }

    impl<T> PoolSectionOps<T> {
        pub fn new() -> PoolSectionOps<T> {
            PoolSectionOps { ops: Vec::new() }
        }
        pub fn insert(&mut self, artifact: T) {
            self.ops.push(PoolSectionOp::Insert(artifact));
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

        fn on_state_change(&self, pool: &ConsensusPoolImpl) -> ChangeSet {
            vec![ChangeAction::AddToValidated(String::from("Consensus message")), ChangeAction::MoveToValidated(String::from("Consensus message"))]
        }
    }
}
