use crate::artifact_manager::ProcessingResult;

pub mod pool;
use crate::consensus_layer::pool::ConsensusPoolImpl;

pub mod consensus;
use crate::consensus_layer::consensus::ConsensusImpl;

pub mod artifacts;
use crate::consensus_layer::artifacts::{ConsensusMessage, UnvalidatedArtifact};

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

    pub fn process_changes(&self, artifacts: Vec<UnvalidatedArtifact<ConsensusMessage>>) -> ProcessingResult {
        if artifacts.len() != 0 {
            {
                println!("Addign artifacts to consensus pool");
                let mut consensus_pool = self.consensus_pool.write().unwrap();
                for artifact in artifacts {
                    consensus_pool.insert(artifact)
                }
            }
            let change_set = {
                let consensus_pool = self.consensus_pool.read().unwrap();
                self.client.on_state_change(&*consensus_pool)
            };
            println!("Change set: {:?}", change_set);
            let changed = if !change_set.is_empty() {
                ProcessingResult::StateChanged
            } else {
                ProcessingResult::StateUnchanged
            };

            self.consensus_pool
                .write()
                .unwrap()
                .apply_changes(change_set);
        }

        ProcessingResult::StateUnchanged
    }
}

