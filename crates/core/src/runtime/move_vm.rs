use move_binary_format::{CompiledModule, errors::VMError};
use move_core_types::{
    account_address::AccountAddress,
    identifier::Identifier,
    language_storage::{ModuleId, TypeTag},
    value::MoveValue,
    vm_status::StatusCode,
};
use move_vm_runtime::{
    move_vm::MoveVM as InnerVM,
    session::Session,
};
use std::sync::Arc;
use crate::protocol::{ProtocolError, ProtocolResult};

/// VM configuration
#[derive(Debug, Clone)]
pub struct VMConfig {
    /// Gas schedule
    pub gas_schedule: Arc<GasSchedule>,
    /// Native functions
    pub native_functions: Arc<NativeFunctions>,
    /// Module publishing options
    pub publishing_options: ModulePublishingOptions,
    /// Max type argument depth
    pub max_type_argument_depth: u8,
    /// Max type nodes
    pub max_type_nodes: u8,
    /// Max dependency depth
    pub max_dependency_depth: u8,
    /// Max function parameters
    pub max_function_parameters: u8,
    /// Max generic instantiation length
    pub max_generic_instantiation_length: u8,
}

impl Default for VMConfig {
    fn default() -> Self {
        Self {
            gas_schedule: Arc::new(GasSchedule::default()),
            native_functions: Arc::new(NativeFunctions::default()),
            publishing_options: ModulePublishingOptions::default(),
            max_type_argument_depth: 32,
            max_type_nodes: 64,
            max_dependency_depth: 100,
            max_function_parameters: 128,
            max_generic_instantiation_length: 32,
        }
    }
}

/// Move VM implementation
pub struct MoveVM {
    /// Inner VM
    inner: InnerVM,
    /// Configuration
    config: VMConfig,
}

impl MoveVM {
    pub fn new(config: VMConfig) -> ProtocolResult<Self> {
        let inner = InnerVM::new(
            config.native_functions.clone(),
            config.gas_schedule.clone(),
        ).map_err(|e| ProtocolError::VMError(e))?;

        Ok(Self { inner, config })
    }

    /// Create new session
    pub fn new_session<'r>(
        &self,
        context: &'r mut ExecutionContext,
    ) -> Session<'r, 'r, ExecutionContext> {
        self.inner.new_session(context)
    }

    /// Execute Move function
    pub async fn execute_function(
        &self,
        module: &ModuleId,
        function: &Identifier,
        ty_args: Vec<TypeTag>,
        args: Vec<Vec<u8>>,
        context: &mut ExecutionContext,
    ) -> ProtocolResult<ExecutionResult> {
        // Verify type arguments
        self.verify_type_arguments(&ty_args)?;

        // Create gas status
        let mut gas_status = GasStatus::new(self.config.gas_schedule.clone());
        
        // Create session
        let mut session = self.new_session(context);
        
        // Execute function
        let result = session.execute_function(
            module,
            function,
            ty_args,
            args,
            &mut gas_status,
        ).map_err(|e| ProtocolError::VMError(e))?;

        // Get events
        let events = session.finish().map_err(|e| ProtocolError::VMError(e))?.1;

        Ok(ExecutionResult {
            return_values: result,
            gas_used: gas_status.remaining_gas(),
            events,
        })
    }

    /// Publish module
    pub async fn publish_module(
        &self,
        module: CompiledModule,
        context: &mut ExecutionContext,
    ) -> ProtocolResult<()> {
        // Verify module
        self.verify_module(&module)?;

        // Check publishing options
        self.config.publishing_options.verify(&module)
            .map_err(|e| ProtocolError::ModulePublishingError(e))?;

        // Create session
        let mut session = self.new_session(context);

        // Publish module
        session.publish_module(
            module.into_inner(),
            AccountAddress::ZERO,
            &mut GasStatus::new(self.config.gas_schedule.clone()),
        ).map_err(|e| ProtocolError::VMError(e))?;

        // Finish session
        session.finish().map_err(|e| ProtocolError::VMError(e))?;

        Ok(())
    }

    /// Verify module
    fn verify_module(&self, module: &CompiledModule) -> ProtocolResult<()> {
        // Verify bytecode
        move_bytecode_verifier::verify_module(module)
            .map_err(|e| ProtocolError::ModuleVerificationError(e))?;

        // Verify module size
        if module.module_handles().len() > self.config.max_dependency_depth as usize {
            return Err(ProtocolError::ModuleVerificationError(
                VMError::new(StatusCode::TOO_MANY_DEPENDENCIES)
            ));
        }

        // Verify dependencies
        self.verify_dependencies(module)?;

        // Verify function parameters
        for func in module.function_defs() {
            if func.parameters.len() > self.config.max_function_parameters as usize {
                return Err(ProtocolError::ModuleVerificationError(
                    VMError::new(StatusCode::TOO_MANY_PARAMETERS)
                ));
            }
        }

        Ok(())
    }

    /// Verify type arguments
    fn verify_type_arguments(&self, ty_args: &[TypeTag]) -> ProtocolResult<()> {
        if ty_args.len() > self.config.max_generic_instantiation_length as usize {
            return Err(ProtocolError::TooManyTypeArguments);
        }

        for ty in ty_args {
            self.verify_type_argument(ty, 0)?;
        }

        Ok(())
    }

    /// Verify single type argument
    fn verify_type_argument(&self, ty: &TypeTag, depth: u8) -> ProtocolResult<()> {
        if depth > self.config.max_type_argument_depth {
            return Err(ProtocolError::TypeArgumentTooDeep);
        }

        match ty {
            TypeTag::Vector(inner) => self.verify_type_argument(inner, depth + 1),
            TypeTag::Struct(struct_tag) => {
                for ty_arg in &struct_tag.type_params {
                    self.verify_type_argument(ty_arg, depth + 1)?;
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }

    /// Verify module dependencies
    fn verify_dependencies(&self, module: &CompiledModule) -> ProtocolResult<()> {
        for dep in module.immediate_dependencies() {
            if !self.inner.has_module(&dep) {
                return Err(ProtocolError::MissingDependency(dep));
            }
        }
        Ok(())
    }
}