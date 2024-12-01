//! Quorum driver module for consensus.

mod driver;

pub use driver::{QuorumDriver, DriverConfig};

use crate::protocol::{ProtocolError, ProtocolResult};

/// Quorum driver error types
#[derive(Debug, thiserror::Error)]
pub enum DriverError {
    #[error("Consensus error: {0}")]
    ConsensusError(String),

    #[error("Certificate error: {0}")]
    CertificateError(String),

    #[error("Timeout error: {0}")]
    TimeoutError(String),

    #[error("Network error: {0}")]
    NetworkError(String),
}

pub type DriverResult<T> = Result<T, DriverError>;

/// Driver status
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DriverStatus {
    /// Driver is active
    Active,
    /// Driver is paused
    Paused,
    /// Driver is stopped
    Stopped,
}