use crate::core::{Object, ObjectID};
use crate::crypto::PublicKey;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

/// Genesis configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenesisConfig {
    /// Chain ID
    pub chain_id: String,
    /// Genesis timestamp
    pub timestamp: u64,
    /// Initial validators
    pub validators: Vec<ValidatorConfig>,
    /// Initial objects
    pub objects: Vec<GenesisObject>,
    /// Framework modules
    pub framework_modules: Vec<FrameworkModule>,
}

/// Validator configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorConfig {
    /// Public key
    pub public_key: PublicKey,
    /// Network address
    pub network_address: String,
    /// Initial stake
    pub stake: u64,
    /// Gas price
    pub gas_price: u64,
}

/// Genesis object
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenesisObject {
    /// Object ID
    pub id: ObjectID,
    /// Owner
    pub owner: String,
    /// Object type
    pub type_: String,
    /// Initial value
    pub value: serde_json::Value,
}

/// Framework module
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameworkModule {
    /// Module name
    pub name: String,
    /// Module bytecode
    pub bytecode: Vec<u8>,
}

impl Default for GenesisConfig {
    fn default() -> Self {
        Self {
            chain_id: "sui-local".to_string(),
            timestamp: 0,
            validators: vec![],
            objects: vec![],
            framework_modules: vec![],
        }
    }
}

/// Genesis state
pub struct Genesis {
    /// Configuration
    config: GenesisConfig,
    /// Objects
    objects: HashMap<ObjectID, Object>,
}

impl Genesis {
    /// Create new genesis state
    pub fn new(config: GenesisConfig) -> Result<Self, GenesisError> {
        let mut objects = HashMap::new();

        // Create objects
        for genesis_object in &config.objects {
            let object = Object::new(
                genesis_object.id,
                genesis_object.owner.clone(),
                genesis_object.type_.clone(),
                genesis_object.value.clone(),
            );
            objects.insert(genesis_object.id, object);
        }

        Ok(Self {
            config,
            objects,
        })
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

    /// Get objects
    pub fn objects(&self) -> &HashMap<ObjectID, Object> {
        &self.objects
    }

    /// Get framework modules
    pub fn framework_modules(&self) -> &[FrameworkModule] {
        &self.config.framework_modules
    }
}

/// Genesis error
#[derive(Debug, thiserror::Error)]
pub enum GenesisError {
    #[error("Invalid object: {0}")]
    InvalidObject(String),

    #[error("Invalid validator: {0}")]
    InvalidValidator(String),

    #[error("Invalid module: {0}")]
    InvalidModule(String),
}