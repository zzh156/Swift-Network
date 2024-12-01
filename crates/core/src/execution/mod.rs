//! Transaction execution module.

mod effects;
mod executor;
mod gas;
mod validator;

pub use effects::{ExecutionEffects, ExecutionStatus};
pub use executor::{Executor, ExecutionContext};
pub use gas::{GasStatus, GasSchedule, GasUnit};
pub use validator::TransactionValidator;

use crate::protocol::{ProtocolError, ProtocolResult};

/// Execution error types
#[derive(Debug, thiserror::Error)]
pub enum ExecutionError {
    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Execution error: {0}")]
    ExecutionError(String),

    #[error("Gas error: {0}")]
    GasError(String),

    #[error("Storage error: {0}")]
    StorageError(String),
}

pub type ExecutionResult<T> = Result<T, ExecutionError>;