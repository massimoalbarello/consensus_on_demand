use std::{collections::BTreeMap};

use super::artifacts::{UnvalidatedArtifact, ConsensusMessage, ChangeSet};

pub struct InMemoryPoolSection {
    artifacts: BTreeMap<String, UnvalidatedArtifact<ConsensusMessage>>,
}

impl InMemoryPoolSection {
    pub fn new() -> InMemoryPoolSection {
        InMemoryPoolSection {
            artifacts: BTreeMap::new(),
        }
    }

    fn mutate(&mut self, ops: PoolSectionOps<UnvalidatedArtifact<ConsensusMessage>>) {
        for op in ops.ops {
            match op {
                PoolSectionOp::Insert(artifact) => self.insert(artifact),
            }
            println!("Inserted artifact in unvalidated section of consensus pool: {:?}", self.artifacts);
        }
    }

    fn insert(&mut self, artifact: UnvalidatedArtifact<ConsensusMessage>) {
        let hash = String::from("Hash");
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

    pub fn insert(&mut self, unvalidated_artifact: UnvalidatedArtifact<ConsensusMessage>) {
        let mut ops = PoolSectionOps::new();
        ops.insert(unvalidated_artifact);
        self.apply_changes_unvalidated(ops);
    }
    
    pub fn apply_changes(&mut self, change_set: ChangeSet) {
        println!("Change set: {:?}", change_set);

    }

    fn apply_changes_unvalidated(&mut self, ops: PoolSectionOps<UnvalidatedArtifact<ConsensusMessage>>) {
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