//! Runtime module for Move VM execution.

pub mod execution;

use crate::protocol::{ProtocolError, ProtocolResult};
use move_vm_runtime::move_vm::MoveVM;
use std::sync::Arc;

/// Runtime configuration
#[derive(Debug, Clone)]
pub struct RuntimeConfig {
    /// Execution configuration
    pub execution: execution::ExecutionConfig,
}

/// Runtime manager
pub struct Runtime {
    /// Configuration
    config: RuntimeConfig,
    /// Move VM
    vm: Arc<MoveVM>,
    /// Execution engine
    execution: execution::ExecutionEngine,
}

impl Runtime {
    pub fn new(config: RuntimeConfig) -> ProtocolResult<Self> {
        let vm = Arc::new(MoveVM::new()?);
        let execution = execution::ExecutionEngine::new(
            config.execution.clone(),
            vm.clone(),
        );

        Ok(Self {
            config,
            vm,
            execution,
        })
    }

    pub fn execution_engine(&self) -> &execution::ExecutionEngine {
        &self.execution
    }

    pub fn vm(&self) -> Arc<MoveVM> {
        self.vm.clone()
    }
}