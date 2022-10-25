use std::{collections::BTreeMap, fmt::Debug};

use crate::crypto::{CryptoHash, CryptoHashOf};

use super::{
    artifacts::{
        UnvalidatedArtifact, ValidatedArtifact, ConsensusMessage, 
        ChangeSet, ChangeAction, IntoInner, 
        ConsensusMessageId, ConsensusMessageHashable
    }, 
    height_index::{
        Indexes, HeightIndexedPool, SelectIndex,
        HeightRange, Height, HeightIndex
    },
    consensus_subcomponents::{
        notary::NotarizationShare,
        aggregator::Notarization,
        block_maker::BlockProposal
    }
};

type UnvalidatedConsensusArtifact = UnvalidatedArtifact<ConsensusMessage>;
type ValidatedConsensusArtifact = ValidatedArtifact<ConsensusMessage>;

pub struct InMemoryPoolSection<T: IntoInner<ConsensusMessage>> {
    pub artifacts: BTreeMap<CryptoHash, T>,
    pub indexes: Indexes,
}

impl<T: IntoInner<ConsensusMessage> + Clone + Debug> InMemoryPoolSection<T> {
    pub fn new() -> InMemoryPoolSection<T> {
        InMemoryPoolSection {
            artifacts: BTreeMap::new(),
            indexes: Indexes::new(),
        }
    }

    fn pool_section(&self) -> &InMemoryPoolSection<T> {
        self
    }

    fn mutate(&mut self, ops: PoolSectionOps<T>) {
        for op in ops.ops {
            match op {
                PoolSectionOp::Insert(artifact) => {
                    println!("Inserting artifact");
                    self.insert(artifact);
                },
                PoolSectionOp::Remove(msg_id) => {
                    if self.remove(&msg_id).is_none() {
                        println!("Error removing artifact {:?}", &msg_id);
                    }
                    else {
                        println!("Removing artifact");
                    }
                }
            }
            
        }
    }

    fn insert(&mut self, artifact: T) {
        let msg = artifact.as_ref();
        let hash = msg.get_cm_hash().digest().clone();
        self.indexes.insert(msg, hash.clone());
        self.artifacts.entry(hash).or_insert(artifact);
    }

    fn remove(&mut self, msg_id: &ConsensusMessageId) -> Option<T> {
        self.remove_by_hash(&msg_id.hash.digest())
    }

    fn get_by_hashes<S: ConsensusMessageHashable>(&self, hashes: Vec<&CryptoHashOf<S>>) -> Vec<S> {
        hashes
            .iter()
            .map(|hash| {
                let artifact_opt = self.get_by_hash(hash.get_ref());
                match artifact_opt {
                    Some(artifact) => match S::assert(artifact.as_ref()) {
                        Some(value) => value.clone(),
                        _ => panic!("Unexpected message type"),
                    },
                    _ => panic!("Can't find artifact with hash: {:?}", hash.get_ref()),
                }
            })
            .collect()
    }

    /// Get a consensus message by its hash
    pub fn get_by_hash(&self, hash: &CryptoHash) -> Option<T> {
        self.artifacts.get(hash).cloned()
    }

    /// Get a consensus message by its hash
    pub fn remove_by_hash(&mut self, hash: &CryptoHash) -> Option<T> {
        self.artifacts.remove(hash).map(|artifact| {
            self.indexes.remove(artifact.as_ref(), hash.to_string());
            artifact
        })
    }

    fn select_index<S: SelectIndex>(&self) -> &HeightIndex<S> {
        SelectIndex::select_index(&self.indexes)
    }

    pub fn notarization_share(&self) -> &dyn HeightIndexedPool<NotarizationShare> {
        self
    }

    pub fn notarization(&self) -> &dyn HeightIndexedPool<Notarization> {
        self
    }

    pub fn block_proposal(&self) -> &dyn HeightIndexedPool<BlockProposal> {
        self
    }
}

impl<
        T: ConsensusMessageHashable + 'static + Debug,
        S: IntoInner<ConsensusMessage> + Clone + Debug,
    > HeightIndexedPool<T> for InMemoryPoolSection<S>
where
    CryptoHashOf<T>: SelectIndex,
{
    fn get_by_height(&self, h: Height) -> Box<dyn Iterator<Item = T>> {
        let hashes = self.select_index().lookup(h).collect();
        // println!("Hashes at height {}: {:?}", h, hashes);
        let artifacts = self.get_by_hashes(hashes);
        // println!("Corresponding artifacts: {:?}", artifacts);
        Box::new(artifacts.into_iter())
    }

    fn height_range(&self) -> Option<HeightRange> {
        let heights = CryptoHashOf::<T>::select_index(&self.indexes)
            .heights()
            .cloned()
            .collect::<Vec<_>>();
        match (heights.first(), heights.last()) {
            (Some(min), Some(max)) => Some(HeightRange::new(*min, *max)),
            _ => None,
        }
    }

    fn max_height(&self) -> Option<Height> {
        self.height_range().map(|range| range.max)
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

    pub fn validated(&self) -> &InMemoryPoolSection<ValidatedConsensusArtifact> {
        self.validated.pool_section()
    }

    pub fn unvalidated(&self) -> &InMemoryPoolSection<UnvalidatedConsensusArtifact> {
        self.unvalidated.pool_section()
    }

    pub fn insert(&mut self, unvalidated_artifact: UnvalidatedConsensusArtifact) {
        let mut ops = PoolSectionOps::new();
        ops.insert(unvalidated_artifact);
        self.apply_changes_unvalidated(ops);
    }
    
    pub fn apply_changes(&mut self, change_set: ChangeSet) {
        let mut unvalidated_ops = PoolSectionOps::new();
        let mut validated_ops = PoolSectionOps::new();

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
                    let msg_id = to_move.get_id();
                    unvalidated_ops.remove(msg_id);
                    validated_ops.insert(ValidatedConsensusArtifact {
                        msg: to_move,
                    });
                }
            }
        }
        self.apply_changes_unvalidated(unvalidated_ops);
        self.apply_changes_validated(validated_ops);
    }

    fn apply_changes_validated(&mut self, ops: PoolSectionOps<ValidatedConsensusArtifact>) {
        if !ops.ops.is_empty() {
            println!("\n########## Consensus pool ##########");
            println!("Applying change to validated section of the consensus pool");
            self.validated.mutate(ops);
        }
    }

    fn apply_changes_unvalidated(&mut self, ops: PoolSectionOps<UnvalidatedConsensusArtifact>) {
        if !ops.ops.is_empty() {
            println!("\n########## Consensus pool ##########");
            println!("Applying change to unvalidated section of the consensus pool");
            self.unvalidated.mutate(ops);
        }
    }
}

#[derive(Debug, Clone)]
pub enum PoolSectionOp<T> {
    Insert(T),
    Remove(ConsensusMessageId),
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

    pub fn remove(&mut self, msg_id: ConsensusMessageId) {
        self.ops.push(PoolSectionOp::Remove(msg_id));
    }
}