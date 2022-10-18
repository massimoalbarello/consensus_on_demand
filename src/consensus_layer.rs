use crate::artifact_manager::ProcessingResult;

pub mod pool;
use crate::consensus_layer::pool::ConsensusPoolImpl;

pub mod consensus;
use crate::consensus_layer::consensus::ConsensusImpl;

pub mod artifacts;
use crate::consensus_layer::artifacts::{ConsensusMessage, UnvalidatedArtifact, ChangeAction};

pub mod pool_reader;

pub mod height_index;

pub mod consensus_subcomponents;

use std::sync::{Arc, RwLock};

pub struct ConsensusProcessor {
    consensus_pool: Arc<RwLock<ConsensusPoolImpl>>,
    client: Box<ConsensusImpl>,
}

impl ConsensusProcessor {
    pub fn new(node_number: u8) -> Self {
        Self {
            consensus_pool: Arc::new(RwLock::new(ConsensusPoolImpl::new())),
            client: Box::new(ConsensusImpl::new(node_number)),
        }
    }

    pub fn process_changes(&self, artifacts: Vec<UnvalidatedArtifact<ConsensusMessage>>) -> (Vec<ConsensusMessage>, ProcessingResult) {
        {
            let mut consensus_pool = self.consensus_pool.write().unwrap();
            for artifact in artifacts {
                consensus_pool.insert(artifact)
            }
        }
        let mut adverts = Vec::new();
        let (change_set, to_broadcast) = {
            let consensus_pool = self.consensus_pool.read().unwrap();
            self.client.on_state_change(&*consensus_pool)
        };
        let changed = if !change_set.is_empty() {
            println!("Change set: {:?} to broadcast: {}", change_set, to_broadcast);
            ProcessingResult::StateChanged
        } else {
            ProcessingResult::StateUnchanged
        };

        if to_broadcast == true {
            for change_action in change_set.iter() {
                match change_action {
                    ChangeAction::AddToValidated(to_add) => {
                        adverts.push(to_add.to_owned());
                    }
                    ChangeAction::MoveToValidated(to_move) => {
                        adverts.push(to_move.to_owned());
                    }
                }
            }
        }

        self.consensus_pool
            .write()
            .unwrap()
            .apply_changes(change_set);

        (adverts, changed)
    }
}

