//! Configuration module for the Sui blockchain.

mod genesis;

pub use genesis::{Genesis, GenesisConfig, GenesisObject};

use serde::{Serialize, Deserialize};
use std::path::PathBuf;

/// Global configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Chain ID
    pub chain_id: String,
    /// Data directory
    pub data_dir: PathBuf,
    /// Genesis configuration
    pub genesis: GenesisConfig,
    /// Network configuration
    pub network: NetworkConfig,
    /// Consensus configuration
    pub consensus: ConsensusConfig,
    /// Authority configuration
    pub authority: AuthorityConfig,
    /// Storage configuration
    pub storage: StorageConfig,
    /// Metrics configuration
    pub metrics: MetricsConfig,
}

/// Network configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// Listen address
    pub listen_address: String,
    /// External address
    pub external_address: String,
    /// Bootstrap nodes
    pub bootstrap_nodes: Vec<String>,
    /// Maximum peers
    pub max_peers: usize,
    /// Connection timeout
    pub connection_timeout_ms: u64,
}

/// Consensus configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusConfig {
    /// Consensus type
    pub consensus_type: ConsensusType,
    /// Block time target (ms)
    pub block_time_ms: u64,
    /// Maximum batch size
    pub max_batch_size: usize,
    /// Narwhal configuration
    pub narwhal: NarwhalConfig,
    /// BullShark configuration
    pub bullshark: BullSharkConfig,
}

/// Consensus type
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ConsensusType {
    /// Narwhal consensus
    Narwhal,
    /// BullShark consensus
    BullShark,
}

/// Authority configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorityConfig {
    /// Authority keypair path
    pub keypair_path: PathBuf,
    /// Network address
    pub network_address: String,
    /// Initial stake
    pub initial_stake: u64,
    /// Gas price
    pub gas_price: u64,
}

/// Storage configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    /// Database path
    pub db_path: PathBuf,
    /// Cache size
    pub cache_size: usize,
    /// Write buffer size
    pub write_buffer_size: usize,
    /// Maximum background jobs
    pub max_background_jobs: i32,
}

/// Metrics configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsConfig {
    /// Enable metrics
    pub enabled: bool,
    /// Listen address
    pub listen_address: String,
}

impl Config {
    /// Load configuration from file
    pub fn load_from_file(path: &str) -> Result<Self, ConfigError> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| ConfigError::IoError(e.to_string()))?;
        
        toml::from_str(&content)
            .map_err(|e| ConfigError::ParseError(e.to_string()))
    }

    /// Save configuration to file
    pub fn save_to_file(&self, path: &str) -> Result<(), ConfigError> {
        let content = toml::to_string_pretty(self)
            .map_err(|e| ConfigError::SerializeError(e.to_string()))?;
        
        std::fs::write(path, content)
            .map_err(|e| ConfigError::IoError(e.to_string()))
    }

    /// Create default configuration
    pub fn default() -> Self {
        Self {
            chain_id: "sui-local".to_string(),
            data_dir: PathBuf::from("data"),
            genesis: GenesisConfig::default(),
            network: NetworkConfig {
                listen_address: "127.0.0.1:8080".to_string(),
                external_address: "127.0.0.1:8080".to_string(),
                bootstrap_nodes: vec![],
                max_peers: 50,
                connection_timeout_ms: 5000,
            },
            consensus: ConsensusConfig {
                consensus_type: ConsensusType::BullShark,
                block_time_ms: 2000,
                max_batch_size: 500,
                narwhal: NarwhalConfig::default(),
                bullshark: BullSharkConfig::default(),
            },
            authority: AuthorityConfig {
                keypair_path: PathBuf::from("key.pem"),
                network_address: "127.0.0.1:8080".to_string(),
                initial_stake: 1_000_000,
                gas_price: 1,
            },
            storage: StorageConfig {
                db_path: PathBuf::from("db"),
                cache_size: 1024 * 1024 * 1024, // 1GB
                write_buffer_size: 64 * 1024 * 1024, // 64MB
                max_background_jobs: 4,
            },
            metrics: MetricsConfig {
                enabled: true,
                listen_address: "127.0.0.1:9184".to_string(),
            },
        }
    }
}

/// Configuration error
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("IO error: {0}")]
    IoError(String),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Serialization error: {0}")]
    SerializeError(String),
}