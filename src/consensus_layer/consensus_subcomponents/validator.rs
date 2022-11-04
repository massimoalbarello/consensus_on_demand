use std::sync::Arc;

use crate::{consensus_layer::{
    pool_reader::PoolReader,
    artifacts::{ChangeSet, ChangeAction, IntoInner},
    consensus::RoundRobin
}, time_source::TimeSource};

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

    pub fn on_state_change(&self, pool_reader: &PoolReader<'_>) -> (ChangeSet, bool) {
        println!("\n########## Validator ##########");
        let validate_artifacts = || self.validate_artifacts(pool_reader);

        let calls: [&'_ dyn Fn() -> (ChangeSet, bool); 1] = [
            &validate_artifacts,
        ];
        self.schedule.call_next(&calls)
    }

    fn validate_artifacts(&self, pool_reader: &PoolReader<'_>) -> (ChangeSet, bool) {
        let mut change_set = Vec::new();
        for (artifact_hash, unvalidated_artifact) in &pool_reader.pool().unvalidated().artifacts {
            println!("Validating artifact {:?}", unvalidated_artifact);
            change_set.push(ChangeAction::MoveToValidated(unvalidated_artifact.to_owned().into_inner()));
        }
        // the changes due to the validation of a block do not have to be broadcasted as each node performs them locally depending on the state of its consensus pool
        (change_set, false)
    }
}