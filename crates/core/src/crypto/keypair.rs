use super::{CryptoError, CryptoResult, SignatureScheme};
use ed25519_dalek::{Keypair as Ed25519Keypair, PublicKey as Ed25519PublicKey, SecretKey};
use rand::rngs::OsRng;
use serde::{Serialize, Deserialize};
use std::fmt;

/// Key pair
#[derive(Clone)]
pub struct KeyPair {
    /// Signature scheme
    scheme: SignatureScheme,
    /// Ed25519 key pair
    ed25519: Option<Ed25519Keypair>,
    /// BLS key pair
    bls: Option<blst::min_pk::SecretKey>,
}

impl KeyPair {
    /// Generate new key pair
    pub fn generate(scheme: SignatureScheme) -> Self {
        match scheme {
            SignatureScheme::Ed25519 => {
                let mut rng = OsRng;
                let keypair = Ed25519Keypair::generate(&mut rng);
                Self {
                    scheme,
                    ed25519: Some(keypair),
                    bls: None,
                }
            }
            SignatureScheme::BLS => {
                let mut rng = OsRng;
                let secret = blst::min_pk::SecretKey::new(&mut rng);
                Self {
                    scheme,
                    ed25519: None,
                    bls: Some(secret),
                }
            }
        }
    }

    /// Create from private key bytes
    pub fn from_private_key_bytes(
        scheme: SignatureScheme,
        bytes: &[u8],
    ) -> CryptoResult<Self> {
        match scheme {
            SignatureScheme::Ed25519 => {
                let secret = SecretKey::from_bytes(bytes)
                    .map_err(|e| CryptoError::InvalidKey(e.to_string()))?;
                let public = Ed25519PublicKey::from(&secret);
                let keypair = Ed25519Keypair { secret, public };
                Ok(Self {
                    scheme,
                    ed25519: Some(keypair),
                    bls: None,
                })
            }
            SignatureScheme::BLS => {
                let secret = blst::min_pk::SecretKey::from_bytes(bytes)
                    .map_err(|e| CryptoError::InvalidKey(e.to_string()))?;
                Ok(Self {
                    scheme,
                    ed25519: None,
                    bls: Some(secret),
                })
            }
        }
    }

    /// Get signature scheme
    pub fn scheme(&self) -> SignatureScheme {
        self.scheme
    }

    /// Get public key
    pub fn public(&self) -> PublicKey {
        match self.scheme {
            SignatureScheme::Ed25519 => {
                let public = self.ed25519.as_ref().unwrap().public;
                PublicKey::Ed25519(public)
            }
            SignatureScheme::BLS => {
                let public = self.bls.as_ref().unwrap().sk_to_pk();
                PublicKey::BLS(public)
            }
        }
    }

    /// Sign message
    pub fn sign(&self, message: &[u8]) -> Signature {
        match self.scheme {
            SignatureScheme::Ed25519 => {
                let signature = self.ed25519.as_ref().unwrap().sign(message);
                Signature::Ed25519(signature)
            }
            SignatureScheme::BLS => {
                let signature = self.bls.as_ref().unwrap().sign(message, &[]);
                Signature::BLS(signature)
            }
        }
    }
}

/// Public key
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PublicKey {
    /// Ed25519 public key
    Ed25519(Ed25519PublicKey),
    /// BLS public key
    BLS(blst::min_pk::PublicKey),
}

impl PublicKey {
    /// Get signature scheme
    pub fn scheme(&self) -> SignatureScheme {
        match self {
            Self::Ed25519(_) => SignatureScheme::Ed25519,
            Self::BLS(_) => SignatureScheme::BLS,
        }
    }

    /// Verify signature
    pub fn verify(&self, message: &[u8], signature: &Signature) -> bool {
        match (self, signature) {
            (Self::Ed25519(pk), Signature::Ed25519(sig)) => {
                pk.verify_strict(message, sig).is_ok()
            }
            (Self::BLS(pk), Signature::BLS(sig)) => {
                sig.verify(true, message, &[], pk, &[]).is_ok()
            }
            _ => false,
        }
    }

    /// Convert to bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            Self::Ed25519(pk) => pk.to_bytes().to_vec(),
            Self::BLS(pk) => pk.to_bytes().to_vec(),
        }
    }
}

impl fmt::Display for PublicKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Ed25519(pk) => write!(f, "ed25519:{}", hex::encode(pk.to_bytes())),
            Self::BLS(pk) => write!(f, "bls:{}", hex::encode(pk.to_bytes())),
        }
    }
}