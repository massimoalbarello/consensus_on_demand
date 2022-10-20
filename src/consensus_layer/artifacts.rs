use serde::{Deserialize, Serialize};

use crate::crypto::ConsensusMessageHash;

use super::consensus_subcomponents::{
    block_maker::BlockProposal,
    notary::NotarizationShare,
    aggregator::Notarization
};

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

pub trait ConsensusMessageHashable: Clone {
    fn get_id(&self) -> ConsensusMessageId;
    fn get_cm_hash(&self) -> ConsensusMessageHash;
    fn assert(msg: &ConsensusMessage) -> Option<&Self>;
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum ConsensusMessage {
    BlockProposal(BlockProposal),
    NotarizationShare(NotarizationShare),
    Notarization(Notarization),
}

impl ConsensusMessageHashable for ConsensusMessage {
    fn get_id(&self) -> ConsensusMessageId {
        ConsensusMessageId {
            hash: self.get_cm_hash(),
            height: 1,
        }
    }

    fn get_cm_hash(&self) -> ConsensusMessageHash {
        match self {
            ConsensusMessage::Notarization(value) => value.get_cm_hash(),
            ConsensusMessage::BlockProposal(value) => value.get_cm_hash(),
            ConsensusMessage::NotarizationShare(value) => value.get_cm_hash(),
        }
    }

    fn assert(msg: &ConsensusMessage) -> Option<&Self> {
        Some(msg)
    }
}

/// Consensus message identifier carries both a message hash and a height,
/// which is used by the consensus pool to help lookup.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ConsensusMessageId {
    pub hash: ConsensusMessageHash,
    pub height: u64,
}

impl ConsensusMessageHashable for BlockProposal {
    fn get_id(&self) -> ConsensusMessageId {
        ConsensusMessageId {
            hash: self.get_cm_hash(),
            height: self.content.value.height,
        }
    }
    
    fn get_cm_hash(&self) -> ConsensusMessageHash {
        ConsensusMessageHash::BlockProposal(self.content.hash.clone())
    }

    fn assert(msg: &ConsensusMessage) -> Option<&Self> {
        if let ConsensusMessage::BlockProposal(value) = msg {
            Some(value)
        } else {
            None
        }
    }
}

impl ConsensusMessageHashable for NotarizationShare {
    fn get_id(&self) -> ConsensusMessageId {
        ConsensusMessageId {
            hash: self.get_cm_hash(),
            height: self.content.height,
        }
    }
    
    fn get_cm_hash(&self) -> ConsensusMessageHash {
        ConsensusMessageHash::NotarizationShare(self.content.block.get_ref().clone())
    }

    fn assert(msg: &ConsensusMessage) -> Option<&Self> {
        if let ConsensusMessage::NotarizationShare(value) = msg {
            Some(value)
        } else {
            None
        }
    }
}

impl ConsensusMessageHashable for Notarization {
    fn get_id(&self) -> ConsensusMessageId {
        ConsensusMessageId {
            hash: self.get_cm_hash(),
            height: self.content.height,
        }
    }
    
    fn get_cm_hash(&self) -> ConsensusMessageHash {
        ConsensusMessageHash::Notarization(self.content.block.get_ref().clone())
    }

    fn assert(msg: &ConsensusMessage) -> Option<&Self> {
        if let ConsensusMessage::Notarization(value) = msg {
            Some(value)
        } else {
            None
        }
    }
}