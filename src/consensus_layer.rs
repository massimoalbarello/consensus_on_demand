use crate::artifact_manager::ProcessingResult;

pub mod pool;
use crate::consensus_layer::pool::ConsensusPoolImpl;

pub mod consensus;
use crate::consensus_layer::consensus::ConsensusImpl;

pub mod artifacts;
use crate::consensus_layer::artifacts::{Artifact, UnvalidatedArtifact};

use std::sync::{Arc, RwLock};

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

