use std::{sync::{Arc, RwLock}, collections::BTreeMap};

use crate::{consensus_layer::{
    pool_reader::PoolReader,
    artifacts::{ChangeSet, ChangeAction, IntoInner, ConsensusMessage},
    consensus::RoundRobin, height_index::Height
}, time_source::TimeSource, HeightMetrics, FinalizationType};

pub struct Validator {
    schedule: RoundRobin,
    time_source: Arc<dyn TimeSource>,
}

impl Validator {
    pub fn new(time_source: Arc<dyn TimeSource>) -> Self {
        Self {
            schedule: RoundRobin::default(),
            time_source,
        }
    }

    pub fn on_state_change(&self, pool_reader: &PoolReader<'_>, finalization_times: Arc<RwLock<BTreeMap<Height, Option<HeightMetrics>>>>) -> (ChangeSet, bool) {
        // println!("\n########## Validator ##########");
        let mut change_set = Vec::new();
        for (artifact_hash, unvalidated_artifact) in &pool_reader.pool().unvalidated().artifacts {
            // println!("Validating artifact {:?}", unvalidated_artifact);
            let consensus_message = unvalidated_artifact.to_owned().into_inner();
            if let ConsensusMessage::Finalization(finalization) = &consensus_message {
                if let Some(finalization_time) =
                        pool_reader.get_finalization_time(finalization.content.height)
                    {
                        let height_metrics = HeightMetrics {
                            latency: finalization_time,
                            fp_finalization: FinalizationType::DK,
                        };

                        // only insert finalization of type DK if received by peer before it was finalized locally
                        if !finalization_times.read().unwrap().contains_key(&finalization.content.height) {
                            finalization_times
                            .write()
                            .unwrap()
                            .insert(finalization.content.height, Some(height_metrics));
                        }
                    }
            }
            change_set.push(ChangeAction::MoveToValidated(consensus_message));
        }
        // the changes due to the validation of a block do not have to be broadcasted as each node performs them locally depending on the state of its consensus pool
        (change_set, false)
    }
}