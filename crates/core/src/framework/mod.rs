//! Framework module for Move smart contracts.

mod abilities;
mod contracts;

pub use abilities::{Ability, ObjectCapabilities};
pub use contracts::{MoveContract, ContractContext};

use crate::protocol::{ProtocolError, ProtocolResult};

/// Framework error types
#[derive(Debug, thiserror::Error)]
pub enum FrameworkError {
    #[error("Contract error: {0}")]
    ContractError(String),

    #[error("Ability error: {0}")]
    AbilityError(String),

    #[error("Execution error: {0}")]
    ExecutionError(String),
}

pub type FrameworkResult<T> = Result<T, FrameworkError>;

/// Framework configuration
#[derive(Debug, Clone)]
pub struct FrameworkConfig {
    /// Maximum type argument depth
    pub max_type_argument_depth: u8,
    /// Maximum function parameters
    pub max_function_parameters: u8,
    /// Maximum generic instantiation length
    pub max_generic_instantiation_length: u8,
    /// Maximum published modules per address
    pub max_modules_per_address: u16,
    /// Maximum dependency depth
    pub max_dependency_depth: u16,
}

impl Default for FrameworkConfig {
    fn default() -> Self {
        Self {
            max_type_argument_depth: 32,
            max_function_parameters: 128,
            max_generic_instantiation_length: 32,
            max_modules_per_address: 256,
            max_dependency_depth: 256,
        }
    }
}