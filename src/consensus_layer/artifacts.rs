use serde::{Deserialize, Serialize};

use super::consensus_subcomponents::{block_maker::BlockProposal, notary::NotarizationShare};

pub type ChangeSet = Vec<ChangeAction>;

#[derive(Debug)]
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
}

impl ConsensusMessage {
    pub fn get_id(&self) -> ConsensusMessageId {
        ConsensusMessageId {
            hash: String::from("Hash"),
            height: 0,
        }
    }
}

/// Consensus message identifier carries both a message hash and a height,
/// which is used by the consensus pool to help lookup.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ConsensusMessageId {
    pub hash: String,
    pub height: u64,
}