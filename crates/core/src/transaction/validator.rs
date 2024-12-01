use super::Transaction;
use crate::core::ObjectID;
use crate::protocol::{ProtocolError, ProtocolResult};
use std::collections::HashSet;

/// Validation result
#[derive(Debug)]
pub struct ValidationResult {
    /// Input objects
    pub input_objects: HashSet<ObjectID>,
    /// Created objects
    pub created_objects: HashSet<ObjectID>,
    /// Modified objects
    pub modified_objects: HashSet<ObjectID>,
    /// Deleted objects
    pub deleted_objects: HashSet<ObjectID>,
}

/// Transaction validator
pub struct TransactionValidator {
    /// Maximum gas budget
    max_gas_budget: u64,
    /// Maximum transaction size
    max_transaction_size: usize,
    /// Maximum input objects
    max_input_objects: usize,
}

impl TransactionValidator {
    /// Create new validator
    pub fn new() -> Self {
        Self {
            max_gas_budget: 1_000_000,
            max_transaction_size: 128 * 1024, // 128KB
            max_input_objects: 2048,
        }
    }

    /// Validate transaction
    pub fn validate_transaction(
        &self,
        transaction: &Transaction,
    ) -> ProtocolResult<ValidationResult> {
        // Validate basic fields
        self.validate_basic_fields(transaction)?;

        // Validate signature
        self.validate_signature(transaction)?;

        // Validate gas
        self.validate_gas(transaction)?;

        // Validate dependencies
        self.validate_dependencies(transaction)?;

        // Validate input objects
        let input_objects = self.validate_input_objects(transaction)?;

        // Create validation result
        Ok(ValidationResult {
            input_objects: input_objects.into_iter().collect(),
            created_objects: HashSet::new(),
            modified_objects: HashSet::new(),
            deleted_objects: HashSet::new(),
        })
    }

    /// Validate basic fields
    fn validate_basic_fields(&self, transaction: &Transaction) -> ProtocolResult<()> {
        // Check transaction size
        let size = bincode::serialize(transaction)
            .map_err(|e| ProtocolError::SerializationError(e.to_string()))?
            .len();
        if size > self.max_transaction_size {
            return Err(ProtocolError::TransactionTooLarge(size));
        }

        // Check sender
        if transaction.sender.is_zero() {
            return Err(ProtocolError::InvalidSender);
        }

        Ok(())
    }

    /// Validate signature
    fn validate_signature(&self, transaction: &Transaction) -> ProtocolResult<()> {
        if !transaction.verify_signature() {
            return Err(ProtocolError::InvalidSignature);
        }
        Ok(())
    }

    /// Validate gas
    fn validate_gas(&self, transaction: &Transaction) -> ProtocolResult<()> {
        if transaction.gas_budget > self.max_gas_budget {
            return Err(ProtocolError::GasBudgetTooHigh);
        }
        if transaction.gas_price == 0 {
            return Err(ProtocolError::InvalidGasPrice);
        }
        Ok(())
    }

    /// Validate dependencies
    fn validate_dependencies(&self, transaction: &Transaction) -> ProtocolResult<()> {
        let mut seen = HashSet::new();
        for dep in &transaction.dependencies {
            if !seen.insert(dep) {
                return Err(ProtocolError::DuplicateDependency(*dep));
            }
        }
        Ok(())
    }

    /// Validate input objects
    fn validate_input_objects(&self, transaction: &Transaction) -> ProtocolResult<Vec<ObjectID>> {
        let input_objects = transaction.input_objects();
        if input_objects.len() > self.max_input_objects {
            return Err(ProtocolError::TooManyInputObjects);
        }
        Ok(input_objects)
    }
}