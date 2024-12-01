use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProtocolError {
    #[error("Invalid transaction: {0}")]
    InvalidTransaction(String),

    #[error("Invalid signature: {0}")]
    InvalidSignature(String),

    #[error("Invalid certificate: {0}")]
    InvalidCertificate(String),

    #[error("Object not found: {0}")]
    ObjectNotFound(String),

    #[error("Version mismatch: expected {expected}, got {actual}")]
    VersionMismatch {
        expected: u64,
        actual: u64,
    },

    #[error("Insufficient gas: required {required}, available {available}")]
    InsufficientGas {
        required: u64,
        available: u64,
    },

    #[error("Transaction expired")]
    TransactionExpired,

    #[error("System error: {0}")]
    SystemError(String),
}

pub type ProtocolResult<T> = Result<T, ProtocolError>;