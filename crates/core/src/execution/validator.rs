use super::{ExecutionError, ExecutionResult};
use crate::core::{Object, ObjectID};
use crate::transaction::{Transaction, TransactionData};
use crate::storage::Storage;
use std::sync::Arc;

/// Transaction validator
pub struct TransactionValidator {
    /// Maximum gas budget
    max_gas_budget: u64,
    /// Maximum transaction size
    max_transaction_size: usize,
    /// Maximum input objects
    max_input_objects: usize,
    /// Maximum created objects
    max_created_objects: usize,
}

impl TransactionValidator {
    /// Create new validator
    pub fn new() -> Self {
        Self {
            max_gas_budget: 1_000_000,
            max_transaction_size: 128 * 1024, // 128KB
            max_input_objects: 2048,
            max_created_objects: 1024,
        }
    }

    /// Validate transaction
    pub fn validate_transaction(
        &self,
        transaction: &Transaction,
    ) -> ExecutionResult<()> {
        // Validate size
        self.validate_transaction_size(transaction)?;

        // Validate gas budget
        self.validate_gas_budget(transaction)?;

        // Validate signature
        self.validate_signature(transaction)?;

        // Validate input objects
        self.validate_input_objects(transaction)?;

        // Validate transaction specific data
        match &transaction.data {
            TransactionData::Move(move_tx) => {
                self.validate_move_transaction(move_tx)?;
            }
            TransactionData::System(system_tx) => {
                self.validate_system_transaction(system_tx)?;
            }
        }

        Ok(())
    }

    /// Validate transaction size
    fn validate_transaction_size(&self, transaction: &Transaction) -> ExecutionResult<()> {
        let size = bincode::serialize(transaction)
            .map_err(|e| ExecutionError::ValidationError(format!("Serialization error: {}", e)))?
            .len();

        if size > self.max_transaction_size {
            return Err(ExecutionError::ValidationError(
                format!("Transaction too large: {} bytes", size)
            ));
        }

        Ok(())
    }

    /// Validate gas budget
    fn validate_gas_budget(&self, transaction: &Transaction) -> ExecutionResult<()> {
        if transaction.gas_budget() > self.max_gas_budget {
            return Err(ExecutionError::ValidationError(
                format!("Gas budget too large: {}", transaction.gas_budget())
            ));
        }

        Ok(())
    }

    /// Validate signature
    fn validate_signature(&self, transaction: &Transaction) -> ExecutionResult<()> {
        if !transaction.verify_signature() {
            return Err(ExecutionError::ValidationError(
                "Invalid transaction signature".into()
            ));
        }

        Ok(())
    }

    /// Validate input objects
    fn validate_input_objects(&self, transaction: &Transaction) -> ExecutionResult<()> {
        let input_objects = transaction.input_objects();

        // Check number of input objects
        if input_objects.len() > self.max_input_objects {
            return Err(ExecutionError::ValidationError(
                format!("Too many input objects: {}", input_objects.len())
            ));
        }

        // Validate each input object
        for object_ref in input_objects {
            self.validate_input_object(object_ref)?;
        }

        Ok(())
    }

    /// Validate input object
    fn validate_input_object(&self, object_ref: &ObjectRef) -> ExecutionResult<()> {
        // Check version
        if object_ref.version == SequenceNumber::MAX {
            return Err(ExecutionError::ValidationError(
                "Invalid object version".into()
            ));
        }

        Ok(())
    }

    /// Validate Move transaction
    fn validate_move_transaction(&self, move_tx: &MoveTransaction) -> ExecutionResult<()> {
        // Validate module
        if let Some(module) = &move_tx.module {
            self.validate_move_module(module)?;
        }

        // Validate function
        if let Some(function) = &move_tx.function {
            self.validate_move_function(function)?;
        }

        // Validate type arguments
        self.validate_type_arguments(&move_tx.type_arguments)?;

        // Validate arguments
        self.validate_arguments(&move_tx.arguments)?;

        Ok(())
    }

    /// Validate Move module
    fn validate_move_module(&self, module: &MoveModule) -> ExecutionResult<()> {
        // Validate bytecode size
        if module.bytecode.len() > self.max_transaction_size {
            return Err(ExecutionError::ValidationError(
                "Module bytecode too large".into()
            ));
        }

        // Validate module dependencies
        for dep in &module.dependencies {
            if !self.is_valid_module_dependency(dep) {
                return Err(ExecutionError::ValidationError(
                    format!("Invalid module dependency: {}", dep)
                ));
            }
        }

        Ok(())
    }

    /// Validate Move function
    fn validate_move_function(&self, function: &MoveFunction) -> ExecutionResult<()> {
        // Validate function name
        if function.name.is_empty() {
            return Err(ExecutionError::ValidationError(
                "Empty function name".into()
            ));
        }

        // Validate visibility
        if !function.is_public() {
            return Err(ExecutionError::ValidationError(
                "Function must be public".into()
            ));
        }

        Ok(())
    }

    /// Validate type arguments
    fn validate_type_arguments(&self, type_args: &[TypeTag]) -> ExecutionResult<()> {
        for type_arg in type_args {
            self.validate_type_argument(type_arg)?;
        }
        Ok(())
    }

    /// Validate type argument
    fn validate_type_argument(&self, type_arg: &TypeTag) -> ExecutionResult<()> {
        match type_arg {
            TypeTag::Struct(struct_tag) => {
                // Validate struct tag
                if struct_tag.module.is_empty() {
                    return Err(ExecutionError::ValidationError(
                        "Empty module name in struct tag".into()
                    ));
                }
                if struct_tag.name.is_empty() {
                    return Err(ExecutionError::ValidationError(
                        "Empty struct name in struct tag".into()
                    ));
                }
                // Recursively validate type parameters
                for type_param in &struct_tag.type_params {
                    self.validate_type_argument(type_param)?;
                }
            }
            TypeTag::Vector(inner) => {
                // Recursively validate inner type
                self.validate_type_argument(inner)?;
            }
            _ => {} // Other primitive types are always valid
        }
        Ok(())
    }

    /// Validate arguments
    fn validate_arguments(&self, arguments: &[Vec<u8>]) -> ExecutionResult<()> {
        for arg in arguments {
            if arg.len() > self.max_transaction_size {
                return Err(ExecutionError::ValidationError(
                    "Argument too large".into()
                ));
            }
        }
        Ok(())
    }

    /// Validate system transaction
    fn validate_system_transaction(&self, system_tx: &SystemTransaction) -> ExecutionResult<()> {
        // Validate system transaction specific rules
        match system_tx {
            SystemTransaction::ChangeEpoch(epoch_change) => {
                self.validate_epoch_change(epoch_change)?;
            }
            SystemTransaction::Genesis(genesis) => {
                self.validate_genesis(genesis)?;
            }
        }

        Ok(())
    }

    /// Validate epoch change
    fn validate_epoch_change(&self, epoch_change: &EpochChange) -> ExecutionResult<()> {
        // Validate epoch number
        if epoch_change.next_epoch == 0 {
            return Err(ExecutionError::ValidationError(
                "Invalid epoch number".into()
            ));
        }

        // Validate validators
        if epoch_change.next_validators.is_empty() {
            return Err(ExecutionError::ValidationError(
                "Empty validator set".into()
            ));
        }

        Ok(())
    }

    /// Validate genesis
    fn validate_genesis(&self, genesis: &Genesis) -> ExecutionResult<()> {
        // Validate timestamp
        if genesis.timestamp == 0 {
            return Err(ExecutionError::ValidationError(
                "Invalid genesis timestamp".into()
            ));
        }

        // Validate initial state
        if genesis.objects.is_empty() {
            return Err(ExecutionError::ValidationError(
                "Empty genesis state".into()
            ));
        }

        Ok(())
    }
}