//! Cryptographic primitives for the Sui blockchain.

mod keypair;
mod signature;

pub use keypair::{KeyPair, PublicKey, PrivateKey};
pub use signature::Signature;

use crate::protocol::{ProtocolError, ProtocolResult};

/// Crypto error types
#[derive(Debug, thiserror::Error)]
pub enum CryptoError {
    #[error("Invalid key: {0}")]
    InvalidKey(String),

    #[error("Invalid signature: {0}")]
    InvalidSignature(String),

    #[error("Invalid scheme: {0}")]
    InvalidScheme(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),
}

pub type CryptoResult<T> = Result<T, CryptoError>;

/// Supported signature schemes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignatureScheme {
    /// Ed25519 signatures
    Ed25519,
    /// BLS signatures
    BLS,
}