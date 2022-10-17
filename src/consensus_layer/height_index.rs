use std::collections::BTreeMap;

use super::artifacts::ConsensusMessage;

pub struct HeightIndex<T: Eq> {
    buckets: BTreeMap<u64, Vec<T>>,
}

impl<T: Eq> Default for HeightIndex<T> {
    fn default() -> Self {
        Self {
            buckets: BTreeMap::new(),
        }
    }
}

/// Provides a thin wrapper around a sorted map of buckets and provides
/// height-indexed access to the buckets.
impl<T: Eq + Clone> HeightIndex<T> {
    pub fn new() -> HeightIndex<T> {
        HeightIndex::default()
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
}

pub struct Indexes {
    pub notarization_share: HeightIndex<String>,
    pub block_proposal: HeightIndex<String>,
}

#[allow(clippy::new_without_default)]
impl Indexes {
    pub fn new() -> Indexes {
        Indexes {
            notarization_share: HeightIndex::new(),
            block_proposal: HeightIndex::new(),
        }
    }

    pub fn remove(&mut self, msg: &ConsensusMessage, hash: &String) {
        match msg {
            ConsensusMessage::NotarizationShare(artifact) => self
                .notarization_share
                .remove(artifact.height, hash),
            ConsensusMessage::BlockProposal(artifact) => self
                .block_proposal
                .remove(artifact.height, hash),
        };
    }
}