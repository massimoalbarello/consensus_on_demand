use super::{pool::ConsensusPoolImpl, artifacts::{ChangeSet, ChangeAction}};

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