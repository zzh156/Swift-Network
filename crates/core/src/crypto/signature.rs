use super::{CryptoError, CryptoResult, PublicKey, SignatureScheme};
use ed25519_dalek::Signature as Ed25519Signature;
use serde::{Serialize, Deserialize};
use std::fmt;

/// Signature
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Signature {
    /// Ed25519 signature
    Ed25519(Ed25519Signature),
    /// BLS signature
    BLS(blst::min_sig::Signature),
}

impl Signature {
    /// Get signature scheme
    pub fn scheme(&self) -> SignatureScheme {
        match self {
            Self::Ed25519(_) => SignatureScheme::Ed25519,
            Self::BLS(_) => SignatureScheme::BLS,
        }
    }

    /// Verify signature
    pub fn verify(&self, message: &[u8], public_key: &PublicKey) -> bool {
        public_key.verify(message, self)
    }

    /// Convert to bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            Self::Ed25519(sig) => sig.to_bytes().to_vec(),
            Self::BLS(sig) => sig.to_bytes().to_vec(),
        }
    }

    /// Create from bytes
    pub fn from_bytes(
        scheme: SignatureScheme,
        bytes: &[u8],
    ) -> CryptoResult<Self> {
        match scheme {
            SignatureScheme::Ed25519 => {
                let sig = Ed25519Signature::from_bytes(bytes)
                    .map_err(|e| CryptoError::InvalidSignature(e.to_string()))?;
                Ok(Self::Ed25519(sig))
            }
            SignatureScheme::BLS => {
                let sig = blst::min_sig::Signature::from_bytes(bytes)
                    .map_err(|e| CryptoError::InvalidSignature(e.to_string()))?;
                Ok(Self::BLS(sig))
            }
        }
    }
}

impl fmt::Display for Signature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Ed25519(sig) => write!(f, "ed25519:{}", hex::encode(sig.to_bytes())),
            Self::BLS(sig) => write!(f, "bls:{}", hex::encode(sig.to_bytes())),
        }
    }
}