use super::{
    ExecutionEffects, ExecutionError, ExecutionResult,
    GasSchedule, GasStatus, TransactionValidator,
};
use crate::core::{Object, ObjectID};
use crate::runtime::{Runtime, RuntimeConfig};
use crate::storage::Storage;
use crate::transaction::{Transaction, TransactionData};
use std::sync::Arc;

/// Execution context
pub struct ExecutionContext {
    /// Storage
    storage: Arc<dyn Storage>,
    /// Gas status
    gas_status: GasStatus,
    /// Modified objects
    modified_objects: Vec<Object>,
    /// Created objects
    created_objects: Vec<Object>,
    /// Deleted objects
    deleted_objects: Vec<ObjectID>,
    /// Events
    events: Vec<Event>,
}

impl ExecutionContext {
    /// Create new execution context
    pub fn new(
        storage: Arc<dyn Storage>,
        gas_schedule: GasSchedule,
        gas_limit: u64,
    ) -> Self {
        Self {
            storage,
            gas_status: GasStatus::new(gas_schedule, gas_limit.into()),
            modified_objects: Vec::new(),
            created_objects: Vec::new(),
            deleted_objects: Vec::new(),
            events: Vec::new(),
        }
    }

    /// Get gas status
    pub fn gas_status(&self) -> &GasStatus {
        &self.gas_status
    }

    /// Get gas status mut
    pub fn gas_status_mut(&mut self) -> &mut GasStatus {
        &mut self.gas_status
    }

    /// Add modified object
    pub fn add_modified_object(&mut self, object: Object) {
        self.modified_objects.push(object);
    }

    /// Add created object
    pub fn add_created_object(&mut self, object: Object) {
        self.created_objects.push(object);
    }

    /// Add deleted object
    pub fn add_deleted_object(&mut self, id: ObjectID) {
        self.deleted_objects.push(id);
    }

    /// Add event
    pub fn add_event(&mut self, event: Event) {
        self.events.push(event);
    }
}

/// Transaction executor
pub struct Executor {
    /// Runtime
    runtime: Arc<Runtime>,
    /// Validator
    validator: Arc<TransactionValidator>,
    /// Storage
    storage: Arc<dyn Storage>,
}

impl Executor {
    /// Create new executor
    pub fn new(
        runtime_config: RuntimeConfig,
        storage: Arc<dyn Storage>,
    ) -> ExecutionResult<Self> {
        let runtime = Runtime::new(runtime_config)
            .map_err(|e| ExecutionError::ExecutionError(e.to_string()))?;
        
        let validator = TransactionValidator::new();

        Ok(Self {
            runtime: Arc::new(runtime),
            validator: Arc::new(validator),
            storage,
        })
    }

    /// Execute transaction
    pub async fn execute_transaction(
        &self,
        transaction: Transaction,
    ) -> ExecutionResult<ExecutionEffects> {
        // Validate transaction
        self.validator.validate_transaction(&transaction)?;

        // Create execution context
        let mut context = ExecutionContext::new(
            self.storage.clone(),
            GasSchedule::default(),
            transaction.gas_budget(),
        );

        // Execute transaction
        let status = match self.execute_transaction_impl(&transaction, &mut context).await {
            Ok(_) => ExecutionStatus::Success,
            Err(e) => ExecutionStatus::Failure {
                error: e.to_string(),
            },
        };

        // Create effects
        let mut effects = ExecutionEffects::new(transaction.digest());
        effects.status = status;
        effects.gas_used = context.gas_status.gas_used().value();

        // Add modified objects
        for object in context.modified_objects {
            effects.add_modified_object(object);
        }

        // Add created objects
        for object in context.created_objects {
            effects.add_created_object(object);
        }

        // Add deleted objects
        for id in context.deleted_objects {
            effects.add_deleted_object(id);
        }

        // Add events
        for event in context.events {
            effects.add_event(event);
        }

        Ok(effects)
    }

    /// Execute transaction implementation
    async fn execute_transaction_impl(
        &self,
        transaction: &Transaction,
        context: &mut ExecutionContext,
    ) -> ExecutionResult<()> {
        match &transaction.data {
            TransactionData::Move(move_tx) => {
                self.runtime.execute_move_transaction(move_tx, context).await
            }
            TransactionData::System(system_tx) => {
                self.runtime.execute_system_transaction(system_tx, context).await
            }
        }
    }
}