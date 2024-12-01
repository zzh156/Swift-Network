//! Authority module for validator node management.

mod authority;
mod authority_store;
mod checkpoint_store;
mod epoch_manager;
mod validator;

pub use authority::{Authority, AuthorityConfig};
pub use authority_store::{AuthorityStore, StoreConfig};
pub use checkpoint_store::{CheckpointStore, Checkpoint};
pub use epoch_manager::{EpochManager, EpochInfo};
pub use validator::{Validator, ValidatorConfig};

use crate::protocol::{ProtocolError, ProtocolResult};
use crate::crypto::KeyPair;
use std::sync::Arc;

/// Authority state
#[derive(Debug, Clone)]
pub struct AuthorityState {
    /// Authority public key
    pub public_key: PublicKey,
    /// Current epoch
    pub epoch: u64,
    /// Stake amount
    pub stake: u64,
    /// Network address
    pub network_address: String,
}

/// Committee information
#[derive(Debug, Clone)]
pub struct CommitteeInfo {
    /// Epoch number
    pub epoch: u64,
    /// Validators
    pub validators: Vec<AuthorityState>,
    /// Quorum threshold
    pub quorum_threshold: u64,
    /// Total stake
    pub total_stake: u64,
}

impl CommitteeInfo {
    /// Check if has quorum
    pub fn has_quorum(&self, stake: u64) -> bool {
        stake >= self.quorum_threshold
    }

    /// Get validator by public key
    pub fn get_validator(&self, public_key: &PublicKey) -> Option<&AuthorityState> {
        self.validators.iter().find(|v| v.public_key == *public_key)
    }

    /// Get total stake
    pub fn total_stake(&self) -> u64 {
        self.total_stake
    }
}

/// Authority error types
#[derive(Debug, thiserror::Error)]
pub enum AuthorityError {
    #[error("Invalid signature")]
    InvalidSignature,

    #[error("Invalid epoch: expected {expected}, got {actual}")]
    InvalidEpoch { expected: u64, actual: u64 },

    #[error("Invalid stake: {0}")]
    InvalidStake(String),

    #[error("Checkpoint error: {0}")]
    CheckpointError(String),

    #[error("Store error: {0}")]
    StoreError(String),
}

pub type AuthorityResult<T> = Result<T, AuthorityError>;