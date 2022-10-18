use crate::consensus_layer::{pool_reader::PoolReader, artifacts::{ChangeSet, ChangeAction, IntoInner}, consensus::RoundRobin};

pub struct Validator {
    schedule: RoundRobin,
}

impl Validator {
    pub fn new() -> Self {
        Self {
            schedule: RoundRobin::default(),
        }
    }

    pub fn on_state_change(&self, pool_reader: &PoolReader<'_>) -> (ChangeSet, bool) {
        let validate_blocks = || self.validate_blocks(pool_reader);

        let calls: [&'_ dyn Fn() -> (ChangeSet, bool); 1] = [
            &validate_blocks,
        ];
        self.schedule.call_next(&calls)
    }

    fn validate_blocks(&self, pool_reader: &PoolReader<'_>) -> (ChangeSet, bool) {
        let mut change_set = Vec::new();
        for (artifact_hash, unvalidated_artifact) in &pool_reader.pool().unvalidated().artifacts {
            println!("Found artifact {:?} in unvalidated section of the consensus pool", unvalidated_artifact);
            change_set.push(ChangeAction::MoveToValidated(unvalidated_artifact.to_owned().into_inner()));
        }
        // the changes due to the validation of a block do not have to be broadcasted as each node performs them locally depending on the state of its consensus pool
        (change_set, false)
    }
}