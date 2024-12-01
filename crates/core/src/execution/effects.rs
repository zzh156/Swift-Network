use crate::core::{Object, ObjectID};
use crate::protocol::Event;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

/// Execution status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecutionStatus {
    /// Success
    Success,
    /// Failure with error message
    Failure { error: String },
}

/// Execution effects
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionEffects {
    /// Transaction digest
    pub transaction_digest: [u8; 32],
    /// Execution status
    pub status: ExecutionStatus,
    /// Gas used
    pub gas_used: u64,
    /// Modified objects
    pub modified_objects: HashMap<ObjectID, Object>,
    /// Created objects
    pub created_objects: HashMap<ObjectID, Object>,
    /// Deleted objects
    pub deleted_objects: Vec<ObjectID>,
    /// Events
    pub events: Vec<Event>,
    /// Dependencies
    pub dependencies: Vec<[u8; 32]>,
}

impl ExecutionEffects {
    /// Create new execution effects
    pub fn new(transaction_digest: [u8; 32]) -> Self {
        Self {
            transaction_digest,
            status: ExecutionStatus::Success,
            gas_used: 0,
            modified_objects: HashMap::new(),
            created_objects: HashMap::new(),
            deleted_objects: Vec::new(),
            events: Vec::new(),
            dependencies: Vec::new(),
        }
    }

    /// Set status
    pub fn set_status(&mut self, status: ExecutionStatus) {
        self.status = status;
    }

    /// Add modified object
    pub fn add_modified_object(&mut self, object: Object) {
        self.modified_objects.insert(object.id(), object);
    }

    /// Add created object
    pub fn add_created_object(&mut self, object: Object) {
        self.created_objects.insert(object.id(), object);
    }

    /// Add deleted object
    pub fn add_deleted_object(&mut self, id: ObjectID) {
        self.deleted_objects.push(id);
    }

    /// Add event
    pub fn add_event(&mut self, event: Event) {
        self.events.push(event);
    }

    /// Add dependency
    pub fn add_dependency(&mut self, dependency: [u8; 32]) {
        self.dependencies.push(dependency);
    }

    /// Set gas used
    pub fn set_gas_used(&mut self, gas_used: u64) {
        self.gas_used = gas_used;
    }

    /// Check if successful
    pub fn is_success(&self) -> bool {
        matches!(self.status, ExecutionStatus::Success)
    }
}