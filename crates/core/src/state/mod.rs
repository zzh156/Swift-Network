//! State management module.

mod accumulator;
mod checkpoint;
mod pruner;
mod store;

pub use accumulator::{StateAccumulator, AccumulatorNode};
pub use checkpoint::{Checkpoint, CheckpointStore};
pub use pruner::{StatePruner, PruneConfig};
pub use store::{StateStore, StateVersion};

use crate::protocol::{ProtocolError, ProtocolResult};

/// State error types
#[derive(Debug, thiserror::Error)]
pub enum StateError {
    #[error("Storage error: {0}")]
    StorageError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Invalid state: {0}")]
    InvalidState(String),

    #[error("Checkpoint error: {0}")]
    CheckpointError(String),
}

pub type StateResult<T> = Result<T, StateError>;