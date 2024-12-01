//! Sui system module for system-level operations.

mod genesis;
mod governance;
mod rewards;
mod stake;
mod validators;

pub use genesis::{Genesis, GenesisConfig};
pub use governance::{Governance, ProposalType, VotingPower};
pub use rewards::{RewardSystem, RewardType};
pub use stake::{StakeSystem, StakeInfo};
pub use validators::{ValidatorSet, ValidatorInfo};

use crate::protocol::{ProtocolError, ProtocolResult};

/// System error types
#[derive(Debug, thiserror::Error)]
pub enum SystemError {
    #[error("Genesis error: {0}")]
    GenesisError(String),

    #[error("Governance error: {0}")]
    GovernanceError(String),

    #[error("Reward error: {0}")]
    RewardError(String),

    #[error("Stake error: {0}")]
    StakeError(String),

    #[error("Validator error: {0}")]
    ValidatorError(String),
}

pub type SystemResult<T> = Result<T, SystemError>;

/// System configuration
#[derive(Debug, Clone)]
pub struct SystemConfig {
    /// Genesis configuration
    pub genesis: GenesisConfig,
    /// Governance configuration
    pub governance: GovernanceConfig,
    /// Reward configuration
    pub rewards: RewardConfig,
    /// Stake configuration
    pub stake: StakeConfig,
    /// Validator configuration
    pub validator: ValidatorConfig,
}

/// System state
pub struct SystemState {
    /// Genesis state
    pub genesis: Genesis,
    /// Governance system
    pub governance: Governance,
    /// Reward system
    pub rewards: RewardSystem,
    /// Stake system
    pub stake: StakeSystem,
    /// Validator set
    pub validators: ValidatorSet,
}

impl SystemState {
    /// Create new system state
    pub fn new(config: SystemConfig) -> SystemResult<Self> {
        Ok(Self {
            genesis: Genesis::new(config.genesis)?,
            governance: Governance::new(config.governance),
            rewards: RewardSystem::new(config.rewards),
            stake: StakeSystem::new(config.stake),
            validators: ValidatorSet::new(config.validator),
        })
    }

    /// Initialize system state
    pub async fn initialize(&mut self) -> SystemResult<()> {
        // Initialize genesis
        self.genesis.initialize().await?;

        // Initialize governance
        self.governance.initialize().await?;

        // Initialize rewards
        self.rewards.initialize().await?;

        // Initialize stake
        self.stake.initialize().await?;

        // Initialize validators
        self.validators.initialize().await?;

        Ok(())
    }
}