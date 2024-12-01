use crate::core::{Object, ObjectID};
use crate::protocol::{ProtocolError, ProtocolResult};
use crate::storage::Storage;
use move_vm_runtime::session::Session;
use std::sync::Arc;

/// Contract context
pub struct ContractContext<'a> {
    /// Storage
    pub storage: Arc<dyn Storage>,
    /// Session
    pub session: &'a mut Session<'a>,
    /// Gas meter
    pub gas_meter: &'a mut GasMeter,
}

/// Move contract
pub struct MoveContract {
    /// Contract address
    address: ObjectID,
    /// Contract module
    module: Vec<u8>,
    /// Contract state
    state: Object,
}

impl MoveContract {
    /// Create new contract
    pub fn new(
        address: ObjectID,
        module: Vec<u8>,
        state: Object,
    ) -> Self {
        Self {
            address,
            module,
            state,
        }
    }

    /// Get contract address
    pub fn address(&self) -> ObjectID {
        self.address
    }

    /// Get contract module
    pub fn module(&self) -> &[u8] {
        &self.module
    }

    /// Get contract state
    pub fn state(&self) -> &Object {
        &self.state
    }

    /// Execute contract function
    pub async fn execute_function(
        &mut self,
        function: &str,
        type_args: Vec<TypeTag>,
        args: Vec<Vec<u8>>,
        context: &mut ContractContext<'_>,
    ) -> ProtocolResult<Vec<Vec<u8>>> {
        // Create module ID
        let module_id = ModuleId::new(
            self.address.into(),
            Identifier::new(function)?,
        );

        // Execute function
        let result = context.session.execute_function(
            &module_id,
            function,
            type_args,
            args,
            context.gas_meter,
        )?;

        // Update contract state
        self.state = context.storage.get_object(&self.address, None)
            .await?
            .ok_or_else(|| ProtocolError::ObjectNotFound(self.address))?;

        Ok(result)
    }

    /// Publish contract module
    pub async fn publish(
        module: Vec<u8>,
        context: &mut ContractContext<'_>,
    ) -> ProtocolResult<Self> {
        // Verify module
        let module_id = verify_module(&module)?;

        // Create contract address
        let address = ObjectID::random();

        // Create initial state
        let state = Object::new_contract_state(address);

        // Publish module
        context.session.publish_module(
            module.clone(),
            address.into(),
            context.gas_meter,
        )?;

        // Store initial state
        context.storage.put_object(&state, None).await?;

        Ok(Self {
            address,
            module,
            state,
        })
    }
}

/// Verify Move module
fn verify_module(module: &[u8]) -> ProtocolResult<ModuleId> {
    // Parse module
    let module = CompiledModule::deserialize(module)
        .map_err(|e| ProtocolError::InvalidModule(e.to_string()))?;

    // Verify bytecode
    move_bytecode_verifier::verify_module(&module)
        .map_err(|e| ProtocolError::InvalidModule(e.to_string()))?;

    Ok(module.self_id())
}