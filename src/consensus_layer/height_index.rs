use std::{collections::BTreeMap, fmt::Debug};

use crate::crypto::{CryptoHash, CryptoHashOf};

use super::{
    artifacts::ConsensusMessage, 
    consensus_subcomponents::{
        aggregator::Notarization,
        block_maker::BlockProposal,
        notary::NotarizationShare
    }
};

#[derive(Debug)]
pub struct HeightIndex<T: Eq + Debug> {
    buckets: BTreeMap<u64, Vec<T>>,
}

impl<T: Eq + Debug> Default for HeightIndex<T> {
    fn default() -> Self {
        Self {
            buckets: BTreeMap::new(),
        }
    }
}

/// Provides a thin wrapper around a sorted map of buckets and provides
/// height-indexed access to the buckets.
impl<T: Eq + Clone + Debug> HeightIndex<T> {
    pub fn new() -> HeightIndex<T> {
        HeightIndex::default()
    }

    /// Inserts `value` at `height`. Returns `true` if `value` was inserted,
    /// `false` if already present.
    pub fn insert(&mut self, height: u64, value: &T) -> bool {
        let values = self.buckets.entry(height).or_insert_with(Vec::new);
        if !values.contains(value) {
            values.push(value.clone());
            return true;
        }
        false
    }

    /// Removes `value` from `height`. Returns `true` if `value` was removed,
    /// `false` if not present.
    pub fn remove(&mut self, height: u64, value: &T) -> bool {
        if let Some(bucket) = self.buckets.get_mut(&height) {
            let len = bucket.len();
            bucket.retain(|x| x != value);
            let removed = len != bucket.len();
            if bucket.is_empty() {
                self.buckets.remove(&height);
            }
            return removed;
        }
        false
    }

    pub fn lookup(&self, height: Height) -> Box<dyn Iterator<Item = &T> + '_> {
        match self.buckets.get(&height) {
            Some(bucket) => Box::new(bucket.iter()),
            None => Box::new(std::iter::empty()),
        }
    }

    /// Returns all heights of the index, in sorted order.
    pub fn heights(&self) -> Box<dyn Iterator<Item = &Height> + '_> {
        Box::new(self.buckets.keys())
    }
}

pub struct Indexes {
    pub notarization_share: HeightIndex<CryptoHashOf<NotarizationShare>>,
    pub block_proposal: HeightIndex<CryptoHashOf<BlockProposal>>,
    pub notarization: HeightIndex<CryptoHashOf<Notarization>>,
}

#[allow(clippy::new_without_default)]
impl Indexes {
    pub fn new() -> Indexes {
        Indexes {
            notarization_share: HeightIndex::new(),
            block_proposal: HeightIndex::new(),
            notarization: HeightIndex::new(),
        }
    }

    pub fn insert(&mut self, msg: &ConsensusMessage, hash: CryptoHash) {
        match msg {
            ConsensusMessage::NotarizationShare(artifact) => {
                self.notarization_share
                    .insert(artifact.content.height, &CryptoHashOf::from(hash))
            },
            ConsensusMessage::BlockProposal(artifact) => {
                self.block_proposal
                    .insert(artifact.content.value.height, &CryptoHashOf::from(hash))
            },
            ConsensusMessage::Notarization(artifact) => {
                self.notarization
                    .insert(artifact.content.height, &CryptoHashOf::from(hash))
            }
        };
    }

    pub fn remove(&mut self, msg: &ConsensusMessage, hash: CryptoHash) {
        match msg {
            ConsensusMessage::NotarizationShare(artifact) => {
                self.notarization_share
                    .remove(artifact.content.height, &CryptoHashOf::from(hash))
            },
            ConsensusMessage::BlockProposal(artifact) => {
                self.block_proposal
                    .remove(artifact.content.value.height, &CryptoHashOf::from(hash))
            },
            ConsensusMessage::Notarization(artifact) => {
                self.notarization
                    .remove(artifact.content.height, &CryptoHashOf::from(hash))
            }
        };
    }
}

/// HeightIndexedPool provides a set of interfaces for the Consensus component
/// to query artifacts. The same interface is applicable to both validated and
/// unvalidated partitions of consensus artifacts in the overall ArtifactPool.
pub trait HeightIndexedPool<T> {
    /// Returns the height range of artifacts of type T currently in the pool.
    fn height_range(&self) -> Option<HeightRange>;

    /// Returns the max height across all artifacts of type T currently in the
    /// pool.
    fn max_height(&self) -> Option<u64>;

    /// Return an iterator over the artifacts of type T at height
    /// 'h'.
    fn get_by_height(&self, h: Height) -> Box<dyn Iterator<Item = T>>;
}

pub trait SelectIndex: Eq + Sized + Debug {
    fn select_index(indexes: &Indexes) -> &HeightIndex<Self>;
}

impl SelectIndex for CryptoHashOf<Notarization> {
    fn select_index(indexes: &Indexes) -> &HeightIndex<Self> {
        &indexes.notarization
    }
}

impl SelectIndex for CryptoHashOf<BlockProposal> {
    fn select_index(indexes: &Indexes) -> &HeightIndex<Self> {
        &indexes.block_proposal
    }
}

impl SelectIndex for CryptoHashOf<NotarizationShare> {
    fn select_index(indexes: &Indexes) -> &HeightIndex<Self> {
        &indexes.notarization_share
    }
}

pub struct HeightRange {
    pub min: Height,
    pub max: Height,
}

impl HeightRange {
    pub fn new(min: Height, max: Height) -> HeightRange {
        HeightRange { min, max }
    }
}

pub type Height = u64;
