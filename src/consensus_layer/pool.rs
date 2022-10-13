use std::{collections::BTreeMap, fmt::Debug};

use crate::consensus_layer::artifacts::ChangeAction;
use super::artifacts::{UnvalidatedArtifact, ValidatedArtifact, ConsensusMessage, ChangeSet, IntoInner};

type UnvalidatedConsensusArtifact = UnvalidatedArtifact<ConsensusMessage>;
type ValidatedConsensusArtifact = ValidatedArtifact<ConsensusMessage>;

pub struct InMemoryPoolSection<T: IntoInner<ConsensusMessage>> {
    artifacts: BTreeMap<String, T>,
}

impl<T: IntoInner<ConsensusMessage> + Clone + Debug> InMemoryPoolSection<T> {
    pub fn new() -> InMemoryPoolSection<T> {
        InMemoryPoolSection {
            artifacts: BTreeMap::new(),
        }
    }

    fn mutate(&mut self, ops: PoolSectionOps<T>) {
        for op in ops.ops {
            match op {
                PoolSectionOp::Insert(artifact) => self.insert(artifact),
            }
            println!("Inserted artifact in unvalidated section of consensus pool: {:?}", self.artifacts);
        }
    }

    fn insert(&mut self, artifact: T) {
        let hash = String::from("Hash");
        self.artifacts.entry(hash).or_insert(artifact);
    }
}

pub struct ConsensusPoolImpl {
    validated: Box<InMemoryPoolSection<ValidatedConsensusArtifact>>,
    unvalidated: Box<InMemoryPoolSection<UnvalidatedConsensusArtifact>>,
}

impl ConsensusPoolImpl {
    pub fn new() -> Self {
        Self {
            validated: Box::new(InMemoryPoolSection::new()),
            unvalidated: Box::new(InMemoryPoolSection::new()),
        }
    }

    pub fn insert(&mut self, unvalidated_artifact: UnvalidatedConsensusArtifact) {
        let mut ops = PoolSectionOps::new();
        ops.insert(unvalidated_artifact);
        self.apply_changes_unvalidated(ops);
    }
    
    pub fn apply_changes(&mut self, change_set: ChangeSet) {
        println!("Change set: {:?}", change_set);
        let mut unvalidated_ops: PoolSectionOps<UnvalidatedConsensusArtifact> = PoolSectionOps::new();
        let mut validated_ops: PoolSectionOps<ValidatedConsensusArtifact> = PoolSectionOps::new();

        // DO NOT Add a default nop. Explicitly mention all cases.
        // This helps with keeping this readable and obvious what
        // change is causing tests to break.
        for change_action in change_set {
            match change_action {
                ChangeAction::AddToValidated(to_add) => {
                    validated_ops.insert(ValidatedConsensusArtifact {
                        msg: to_add,
                    });
                }
                ChangeAction::MoveToValidated(to_move) => {
                    validated_ops.insert(ValidatedConsensusArtifact {
                        msg: to_move,
                    });
                }
            }
        }
    }

    fn apply_changes_validated(&mut self, ops: PoolSectionOps<ValidatedConsensusArtifact>) {
        if !ops.ops.is_empty() {
            self.validated.mutate(ops);
        }
    }

    fn apply_changes_unvalidated(&mut self, ops: PoolSectionOps<UnvalidatedConsensusArtifact>) {
        if !ops.ops.is_empty() {
            self.unvalidated.mutate(ops);
        }
    }

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