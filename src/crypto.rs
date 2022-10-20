use serde::{Serialize, Deserialize};
use sha2::{Digest, Sha256};
use std::marker::PhantomData;

use crate::consensus_layer::consensus_subcomponents::block_maker::Block;

// Signed contains the signed content and its signature.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Signed<T, S> {
    pub content: T,
    pub signature: S,
}

/// Bundle of both a value and its hash. Once created it remains immutable,
/// which is why both fields are only accessible through member functions, not
/// as record fields.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Hashed {
    pub(crate) hash: String,
    pub(crate) value: Block,
}

impl Hashed {
    pub fn new(artifact: Block) -> Self {
        Self {
            hash: Hashed::calculate_hash(&artifact),
            value: artifact
        }
    }

    fn calculate_hash(artifact: &Block) -> CryptoHash {
        let payload = serde_json::json!(artifact);
        let mut hasher = Sha256::new();
        hasher.update(payload.to_string().as_bytes());
        hex::encode(hasher.finalize().as_slice().to_owned())
    }
}

pub type CryptoHash = String;

/// A cryptographic hash for content of type `T`
pub type CryptoHashOf<T> = Id<T, CryptoHash>;

#[derive(Eq, PartialEq, Clone)]
pub struct Id<Entity, Repr>(Repr, PhantomData<Entity>);

impl<Entity, Repr> Id<Entity, Repr> {
    pub const fn new(repr: Repr) -> Id<Entity, Repr> {
        Id(repr, PhantomData)
    }
}

impl<Entity, Repr> From<Repr> for Id<Entity, Repr> {
    fn from(repr: Repr) -> Self {
        Self::new(repr)
    }
}

/// ConsensusMessageHash has the same variants as [ConsensusMessage], but
/// contains only a hash instead of the full message in each variant.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ConsensusMessageHash {
    NotarizationShare(CryptoHash),
    BlockProposal(CryptoHash),
    Notarization(CryptoHash),
}

impl ConsensusMessageHash {
    pub fn digest(&self) -> &CryptoHash {
        match self {
            ConsensusMessageHash::NotarizationShare(hash) => hash,
            ConsensusMessageHash::BlockProposal(hash) => hash,
            ConsensusMessageHash::Notarization(hash) => hash,
        }
    }
}