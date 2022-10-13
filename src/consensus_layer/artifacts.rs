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

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum ConsensusMessage {
    BlockProposal(BlockProposal),
    NotarizationShare(NotarizationShare),
}