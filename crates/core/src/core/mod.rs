//! Core types and data structures for the Sui blockchain.

mod object;
mod types;

pub use object::{Object, ObjectID, Owner};
pub use types::{Address, Balance, Coin, SequenceNumber, TypeTag};

use serde::{Serialize, Deserialize};
use std::fmt;

/// Core error types
#[derive(Debug, thiserror::Error)]
pub enum CoreError {
    #[error("Invalid object: {0}")]
    InvalidObject(String),

    #[error("Invalid address: {0}")]
    InvalidAddress(String),

    #[error("Invalid type: {0}")]
    InvalidType(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),
}

pub type CoreResult<T> = Result<T, CoreError>;