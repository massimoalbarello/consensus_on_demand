use serde::{Deserialize, Serialize};

use crate::{crypto::{ConsensusMessageHash, Hashed}, time_source::Time};

use super::consensus_subcomponents::{
    block_maker::BlockProposal,
    notary::{NotarizationShare, NotarizationShareContent},
    aggregator::{Notarization, Finalization}, finalizer::FinalizationShare
};

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

pub trait HasTimestamp {
    fn timestamp(&self) -> Time;
}

// Unvalidated artifact
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnvalidatedArtifact<T> {
    pub message: T,
    pub peer_id: u8,
    pub timestamp: Time,
}

impl<T> UnvalidatedArtifact<T> {
    pub fn new(artifact: T, timestamp: Time) -> Self {
        Self {
            message: artifact,
            peer_id: 0,
            timestamp,
        }
    }
}

impl<T> IntoInner<T> for UnvalidatedArtifact<T> {
    fn into_inner(self) -> T {
        self.message
    }
}

impl<T> HasTimestamp for UnvalidatedArtifact<T> {
    fn timestamp(&self) -> Time {
        self.timestamp
    }
}

// Validated artifact
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidatedArtifact<T> {
    pub msg: T,
    pub timestamp: Time,
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

impl<T> HasTimestamp for ValidatedArtifact<T> {
    fn timestamp(&self) -> Time {
        self.timestamp
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
    FinalizationShare(FinalizationShare),
    Finalization(Finalization),

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
            ConsensusMessage::BlockProposal(value) => value.get_cm_hash(),
            ConsensusMessage::NotarizationShare(value) => value.get_cm_hash(),
            ConsensusMessage::Notarization(value) => value.get_cm_hash(),
            ConsensusMessage::FinalizationShare(value) => value.get_cm_hash(),
            ConsensusMessage::Finalization(value) => value.get_cm_hash(),
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
        ConsensusMessageHash::BlockProposal(Hashed::crypto_hash(self))
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
        match self.content.to_owned() {
            NotarizationShareContent::COD(share_content) => {
                ConsensusMessageId {
                    hash: self.get_cm_hash(),
                    height: share_content.height,
                }
            },
            NotarizationShareContent::ICC(share_content) => {
                ConsensusMessageId {
                    hash: self.get_cm_hash(),
                    height: share_content.height,
                }
            }
        }
    }
    
    fn get_cm_hash(&self) -> ConsensusMessageHash {
        ConsensusMessageHash::NotarizationShare(Hashed::crypto_hash(self))
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
        ConsensusMessageHash::Notarization(Hashed::crypto_hash(self))
    }
    
    fn assert(msg: &ConsensusMessage) -> Option<&Self> {
        if let ConsensusMessage::Notarization(value) = msg {
            Some(value)
        } else {
            None
        }
    }
}

impl ConsensusMessageHashable for FinalizationShare {
    fn get_id(&self) -> ConsensusMessageId {
        ConsensusMessageId {
            hash: self.get_cm_hash(),
            height: self.content.height,
        }
    }
    
    fn get_cm_hash(&self) -> ConsensusMessageHash {
        ConsensusMessageHash::FinalizationShare(Hashed::crypto_hash(self))
    }

    fn assert(msg: &ConsensusMessage) -> Option<&Self> {
        if let ConsensusMessage::FinalizationShare(value) = msg {
            Some(value)
        } else {
            None
        }
    }
}

impl ConsensusMessageHashable for Finalization {
    fn get_id(&self) -> ConsensusMessageId {
        ConsensusMessageId {
            hash: self.get_cm_hash(),
            height: self.content.height,
        }
    }
    
    fn get_cm_hash(&self) -> ConsensusMessageHash {
        ConsensusMessageHash::Finalization(Hashed::crypto_hash(self))
    }

    fn assert(msg: &ConsensusMessage) -> Option<&Self> {
        if let ConsensusMessage::Finalization(value) = msg {
            Some(value)
        } else {
            None
        }
    }
}