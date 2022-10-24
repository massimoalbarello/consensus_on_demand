use serde::{Serialize, Deserialize};
use sha2::{Digest, Sha256};
use std::{marker::PhantomData, hash::Hash};

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
pub struct Hashed<T> {
    pub(crate) hash: CryptoHash,
    pub(crate) value: T,
}

impl<T> Hashed<T> {
    pub fn new(artifact: T) -> Self 
    where T: Serialize 
    {
        Self {
            hash: Hashed::crypto_hash(&artifact),
            value: artifact
        }
    }

    /// Return the hash field as reference.
    pub fn get_hash(&self) -> &CryptoHash {
        &self.hash
    }

    pub fn crypto_hash(artifact: &T) -> CryptoHash 
    where T: Serialize
    {
        let payload = serde_json::json!(artifact);
        let mut hasher = Sha256::new();
        hasher.update(payload.to_string().as_bytes());
        hex::encode(hasher.finalize().as_slice().to_owned())
    }
}

pub type CryptoHash = String;

/// A cryptographic hash for content of type `T`
pub type CryptoHashOf<T> = Id<T, CryptoHash>;

#[derive(Eq, PartialEq, Clone, Debug, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Id<Entity, Repr>(Repr, PhantomData<Entity>);

impl<Entity, Repr> Id<Entity, Repr> {
    pub const fn new(repr: Repr) -> Id<Entity, Repr> {
        Id(repr, PhantomData)
    }

    pub const fn get_ref(&self) -> &Repr {
        &self.0
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