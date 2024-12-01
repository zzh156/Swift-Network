use crate::protocol::{ProtocolError, ProtocolResult};
use move_core_types::{
    account_address::AccountAddress,
    effects::Event,
    identifier::Identifier,
    language_storage::{ModuleId, StructTag},
    value::MoveValue,
};
use std::collections::HashMap;

/// Execution context
pub struct ExecutionContext {
    /// State view
    state: StateView,
    /// Events
    events: Vec<Event>,
    /// Published modules
    modules: HashMap<ModuleId, Vec<u8>>,
    /// Resources
    resources: HashMap<(AccountAddress, StructTag), Vec<u8>>,
}

impl ExecutionContext {
    pub fn new(state: StateView) -> Self {
        Self {
            state,
            events: Vec::new(),
            modules: HashMap::new(),
            resources: HashMap::new(),
        }
    }

    /// Get state view
    pub fn state(&self) -> &StateView {
        &self.state
    }

    /// Get mutable state view
    pub fn state_mut(&mut self) -> &mut StateView {
        &mut self.state
    }

    /// Get events
    pub fn events(&self) -> &[Event] {
        &self.events
    }

    /// Add event
    pub fn add_event(&mut self, event: Event) {
        self.events.push(event);
    }

    /// Get module
    pub fn get_module(&self, id: &ModuleId) -> Option<&[u8]> {
        self.modules.get(id).map(|v| v.as_slice())
    }

    /// Add module
    pub fn add_module(&mut self, id: ModuleId, module: Vec<u8>) {
        self.modules.insert(id, module);
    }

    /// Get resource
    pub fn get_resource(
        &self,
        address: AccountAddress,
        tag: StructTag,
    ) -> Option<&[u8]> {
        self.resources.get(&(address, tag)).map(|v| v.as_slice())
    }

    /// Set resource
    pub fn set_resource(
        &mut self,
        address: AccountAddress,
        tag: StructTag,
        value: Vec<u8>,
    ) {
        self.resources.insert((address, tag), value);
    }

    /// Delete resource
    pub fn delete_resource(
        &mut self,
        address: AccountAddress,
        tag: StructTag,
    ) -> Option<Vec<u8>> {
        self.resources.remove(&(address, tag))
    }
}

/// Execution result
#[derive(Debug)]
pub struct ExecutionResult {
    /// Return values
    pub return_values: Vec<MoveValue>,
    /// Events
    pub events: Vec<Event>,
}

impl ExecutionResult {
    pub fn new(return_values: Vec<MoveValue>, events: Vec<Event>) -> Self {
        Self {
            return_values,
            events,
        }
    }
}