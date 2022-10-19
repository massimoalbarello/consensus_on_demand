use serde::{Deserialize, Serialize};

use crate::crypto::ConsensusMessageHash;

use super::consensus_subcomponents::{block_maker::BlockProposal, notary::NotarizationShare, aggregator::Notarization};

pub const N: usize = 4;

pub type ChangeSet = Vec<ChangeAction>;

#[derive(Debug, Clone)]
pub enum ChangeAction {
    AddToValidated(ConsensusMessage),
    MoveToValidated(ConsensusMessage),
}

impl From<ChangeAction> for ChangeSet {
    fn from(action: ChangeAction) -> Self {
        vec![action]
    }
}

/// A trait similar to Into, but without its restrictions.
pub trait IntoInner<T>: AsRef<T> {
    fn into_inner(self) -> T;
}

impl<T> AsRef<T> for UnvalidatedArtifact<T> {
    fn as_ref(&self) -> &T {
        &self.message
    }
}

// Unvalidated artifact
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnvalidatedArtifact<T> {
    pub message: T,
    pub peer_id: u8,
}

impl<T> UnvalidatedArtifact<T> {
    pub fn new(artifact: T) -> Self {
        Self {
            message: artifact,
            peer_id: 0,
        }
    }
}

impl<T> IntoInner<T> for UnvalidatedArtifact<T> {
    fn into_inner(self) -> T {
        self.message
    }
}

// Validated artifact
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidatedArtifact<T> {
    pub msg: T,
}

impl<T> IntoInner<T> for ValidatedArtifact<T> {
    fn into_inner(self) -> T {
        self.msg
    }
}

impl<T> AsRef<T> for ValidatedArtifact<T> {
    fn as_ref(&self) -> &T {
        &self.msg
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum ConsensusMessage {
    BlockProposal(BlockProposal),
    NotarizationShare(NotarizationShare),
    Notarization(Notarization),
}

impl ConsensusMessage {
    pub fn get_id(&self) -> ConsensusMessageId {
        ConsensusMessageId {
            hash: self.get_cm_hash(),
            height: 0,
        }
    }
    
    pub fn get_cm_hash(&self) -> ConsensusMessageHash {
        match self {
            ConsensusMessage::BlockProposal(artifact) => ConsensusMessageHash::BlockProposal(artifact.content.hash.clone()),
            ConsensusMessage::NotarizationShare(artifact) => ConsensusMessageHash::NotarizationShare(artifact.content.block.clone()),
            ConsensusMessage::Notarization(artifact) => ConsensusMessageHash::Notarization(artifact.content.block.clone()),
        }
    }
}

/// Consensus message identifier carries both a message hash and a height,
/// which is used by the consensus pool to help lookup.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ConsensusMessageId {
    pub hash: ConsensusMessageHash,
    pub height: u64,
}