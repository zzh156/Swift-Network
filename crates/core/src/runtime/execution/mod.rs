//! Execution module for transaction processing.

mod context;

pub use context::{ExecutionContext, ExecutionResult};

use crate::protocol::{ProtocolError, ProtocolResult};
use move_vm_runtime::session::Session;

/// Execution engine configuration
#[derive(Debug, Clone)]
pub struct ExecutionConfig {
    /// Maximum gas per transaction
    pub max_gas_per_tx: u64,
    /// Maximum number of events
    pub max_events: usize,
}

/// Execution engine
pub struct ExecutionEngine {
    /// Configuration
    config: ExecutionConfig,
    /// Move VM
    vm: Arc<MoveVM>,
}

impl ExecutionEngine {
    pub fn new(config: ExecutionConfig, vm: Arc<MoveVM>) -> Self {
        Self { config, vm }
    }

    /// Execute transaction
    pub async fn execute_transaction(
        &self,
        tx: SignedTransaction,
        context: &mut ExecutionContext,
    ) -> ProtocolResult<ExecutionResult> {
        // Create new session
        let session = self.vm.new_session(context);

        // Execute transaction
        let result = match tx.payload {
            TransactionPayload::Script(script) => {
                self.execute_script(script, session, context).await
            }
            TransactionPayload::ModuleBundle(modules) => {
                self.publish_modules(modules, session, context).await
            }
            TransactionPayload::Function(function) => {
                self.execute_function(function, session, context).await
            }
        }?;

        Ok(result)
    }

    /// Execute script
    async fn execute_script(
        &self,
        script: Script,
        session: Session<ExecutionContext>,
        context: &mut ExecutionContext,
    ) -> ProtocolResult<ExecutionResult> {
        // Verify script
        self.vm.verify_script(&script)?;

        // Execute script
        let result = session.execute_script(
            script.code,
            script.ty_args,
            script.args,
            &mut GasStatus::new(self.config.max_gas_per_tx),
        )?;

        Ok(ExecutionResult::new(result, context.events().to_vec()))
    }

    /// Publish modules
    async fn publish_modules(
        &self,
        modules: Vec<Module>,
        session: Session<ExecutionContext>,
        context: &mut ExecutionContext,
    ) -> ProtocolResult<ExecutionResult> {
        // Verify modules
        for module in &modules {
            self.vm.verify_module(module)?;
        }

        // Publish modules
        for module in modules {
            session.publish_module(
                module.code,
                module.sender,
                &mut GasStatus::new(self.config.max_gas_per_tx),
            )?;
        }

        Ok(ExecutionResult::new(vec![], context.events().to_vec()))
    }

    /// Execute function
    async fn execute_function(
        &self,
        function: Function,
        session: Session<ExecutionContext>,
        context: &mut ExecutionContext,
    ) -> ProtocolResult<ExecutionResult> {
        let result = session.execute_function(
            &function.module,
            &function.function,
            function.ty_args,
            function.args,
            &mut GasStatus::new(self.config.max_gas_per_tx),
        )?;

        Ok(ExecutionResult::new(result, context.events().to_vec()))
    }
}