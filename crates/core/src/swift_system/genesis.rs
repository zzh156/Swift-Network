use super::{SystemError, SystemResult};
use crate::core::{Object, ObjectID};
use crate::storage::Storage;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

/// Genesis configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenesisConfig {
    /// Chain ID
    pub chain_id: String,
    /// Genesis timestamp
    pub timestamp: u64,
    /// Initial validators
    pub validators: Vec<ValidatorConfig>,
    /// Framework objects
    pub framework_objects: Vec<FrameworkObject>,
}

/// Framework object
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameworkObject {
    /// Object ID
    pub id: ObjectID,
    /// Object type
    pub type_: String,
    /// Initial value
    pub value: serde_json::Value,
}

/// Genesis state
pub struct Genesis {
    /// Configuration
    config: GenesisConfig,
    /// Storage
    storage: Arc<dyn Storage>,
    /// Initialized flag
    initialized: bool,
}

impl Genesis {
    /// Create new genesis
    pub fn new(
        config: GenesisConfig,
        storage: Arc<dyn Storage>,
    ) -> SystemResult<Self> {
        Ok(Self {
            config,
            storage,
            initialized: false,
        })
    }

    /// Initialize genesis state
    pub async fn initialize(&mut self) -> SystemResult<()> {
        if self.initialized {
            return Err(SystemError::GenesisError("Already initialized".into()));
        }

        // Create framework objects
        for object in &self.config.framework_objects {
            let obj = Object::new(
                object.id,
                object.type_.clone(),
                object.value.clone(),
            );
            self.storage.put_object(&obj, None).await
                .map_err(|e| SystemError::GenesisError(e.to_string()))?;
        }

        // Initialize validators
        for validator in &self.config.validators {
            self.initialize_validator(validator).await?;
        }

        self.initialized = true;
        Ok(())
    }

    /// Initialize validator
    async fn initialize_validator(&self, config: &ValidatorConfig) -> SystemResult<()> {
        // Create validator object
        let validator = Object::new_validator(
            ObjectID::random(),
            config.public_key.clone(),
            config.network_address.clone(),
            config.stake_amount,
        );

        // Store validator
        self.storage.put_object(&validator, None).await
            .map_err(|e| SystemError::GenesisError(e.to_string()))?;

        Ok(())
    }

    /// Get chain ID
    pub fn chain_id(&self) -> &str {
        &self.config.chain_id
    }

    /// Get timestamp
    pub fn timestamp(&self) -> u64 {
        self.config.timestamp
    }

    /// Get validators
    pub fn validators(&self) -> &[ValidatorConfig] {
        &self.config.validators
    }

    /// Get framework objects
    pub fn framework_objects(&self) -> &[FrameworkObject] {
        &self.config.framework_objects
    }

    /// Check if initialized
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }
}