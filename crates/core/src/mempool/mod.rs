//! Mempool module for transaction management and prioritization.

mod pool;
mod prioritizer;

pub use pool::{Mempool, MempoolConfig};
pub use prioritizer::{Priority, TransactionPrioritizer};

use crate::protocol::{SignedTransaction, TransactionDigest};

/// Mempool errors
#[derive(Debug, thiserror::Error)]
pub enum MempoolError {
    #[error("Transaction already exists")]
    DuplicateTransaction,

    #[error("Mempool is full")]
    MempoolFull,

    #[error("Invalid transaction: {0}")]
    InvalidTransaction(String),

    #[error("Transaction expired")]
    TransactionExpired,
}

pub type MempoolResult<T> = Result<T, MempoolError>;