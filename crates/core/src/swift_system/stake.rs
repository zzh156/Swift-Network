use super::{SystemError, SystemResult};
use crate::core::{Address, ObjectID};
use crate::storage::Storage;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Stake configuration
#[derive(Debug, Clone)]
pub struct StakeConfig {
    /// Minimum stake amount
    pub min_stake_amount: u64,
    /// Maximum stake amount
    pub max_stake_amount: u64,
    /// Minimum stake duration
    pub min_stake_duration: u64,
    /// Maximum stake duration
    pub max_stake_duration: u64,
    /// Unstake delay
    pub unstake_delay: u64,
}

/// Stake info
#[derive(Debug, Clone)]
pub struct StakeInfo {
    /// Stake ID
    pub id: ObjectID,
    /// Staker address
    pub staker: Address,
    /// Stake amount
    pub amount: u64,
    /// Start time
    pub start_time: u64,
    /// Duration
    pub duration: u64,
    /// Status
    pub status: StakeStatus,
}

/// Stake status
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StakeStatus {
    /// Active
    Active,
    /// Unstaking
    Unstaking {
        /// Unstake time
        unstake_time: u64,
    },
    /// Withdrawn
    Withdrawn,
}

/// Stake system
pub struct StakeSystem {
    /// Configuration
    config: StakeConfig,
    /// Storage
    storage: Arc<dyn Storage>,
    /// Stakes
    stakes: RwLock<HashMap<ObjectID, StakeInfo>>,
    /// Total staked amount
    total_staked: RwLock<u64>,
}

impl StakeSystem {
    /// Create new stake system
    pub fn new(
        config: StakeConfig,
        storage: Arc<dyn Storage>,
    ) -> Self {
        Self {
            config,
            storage,
            stakes: RwLock::new(HashMap::new()),
            total_staked: RwLock::new(0),
        }
    }

    /// Initialize stake system
    pub async fn initialize(&mut self) -> SystemResult<()> {
        // Load stakes
        let stakes = self.storage.get_stakes().await
            .map_err(|e| SystemError::StakeError(e.to_string()))?;
        *self.stakes.write().await = stakes;

        // Calculate total staked
        let total: u64 = stakes.values()
            .filter(|stake| stake.status == StakeStatus::Active)
            .map(|stake| stake.amount)
            .sum();
        *self.total_staked.write().await = total;

        Ok(())
    }

    /// Create stake
    pub async fn create_stake(
        &self,
        staker: Address,
        amount: u64,
        duration: u64,
    ) -> SystemResult<ObjectID> {
        // Validate amount
        if amount < self.config.min_stake_amount || amount > self.config.max_stake_amount {
            return Err(SystemError::StakeError("Invalid stake amount".into()));
        }

        // Validate duration
        if duration < self.config.min_stake_duration || duration > self.config.max_stake_duration {
            return Err(SystemError::StakeError("Invalid stake duration".into()));
        }

        // Create stake info
        let stake = StakeInfo {
            id: ObjectID::random(),
            staker,
            amount,
            start_time: crate::utils::current_timestamp(),
            duration,
            status: StakeStatus::Active,
        };

        // Store stake
        self.storage.put_stake(&stake).await
            .map_err(|e| SystemError::StakeError(e.to_string()))?;
        self.stakes.write().await.insert(stake.id, stake.clone());

        // Update total staked
        *self.total_staked.write().await += amount;

        Ok(stake.id)
    }

    /// Start unstaking
    pub async fn start_unstake(&self, stake_id: ObjectID) -> SystemResult<()> {
        // Get stake
        let mut stakes = self.stakes.write().await;
        let stake = stakes.get_mut(&stake_id)
            .ok_or_else(|| SystemError::StakeError("Stake not found".into()))?;

        // Check status
        if stake.status != StakeStatus::Active {
            return Err(SystemError::StakeError("Stake not active".into()));
        }

        // Check minimum duration
        let elapsed = crate::utils::current_timestamp() - stake.start_time;
        if elapsed < stake.duration {
            return Err(SystemError::StakeError("Minimum duration not met".into()));
        }

        // Update status
        stake.status = StakeStatus::Unstaking {
            unstake_time: crate::utils::current_timestamp(),
        };

        // Store updated stake
        self.storage.put_stake(stake).await
            .map_err(|e| SystemError::StakeError(e.to_string()))?;

        Ok(())
    }

    /// Withdraw stake
    pub async fn withdraw_stake(&self, stake_id: ObjectID) -> SystemResult<u64> {
        // Get stake
        let mut stakes = self.stakes.write().await;
        let stake = stakes.get_mut(&stake_id)
            .ok_or_else(|| SystemError::StakeError("Stake not found".into()))?;

        // Check status
        let unstake_time = match stake.status {
            StakeStatus::Unstaking { unstake_time } => unstake_time,
            _ => return Err(SystemError::StakeError("Stake not unstaking".into())),
        };

        // Check delay
        let elapsed = crate::utils::current_timestamp() - unstake_time;
        if elapsed < self.config.unstake_delay {
            return Err(SystemError::StakeError("Unstake delay not met".into()));
        }

        // Update status
        stake.status = StakeStatus::Withdrawn;

        // Store updated stake
        self.storage.put_stake(stake).await
            .map_err(|e| SystemError::StakeError(e.to_string()))?;

        // Update total staked
        *self.total_staked.write().await -= stake.amount;

        Ok(stake.amount)
    }

    /// Get stake info
    pub async fn get_stake(&self, stake_id: &ObjectID) -> SystemResult<Option<StakeInfo>> {
        Ok(self.stakes.read().await.get(stake_id).cloned())
    }

    /// Get stakes by staker
    pub async fn get_stakes_by_staker(&self, staker: &Address) -> SystemResult<Vec<StakeInfo>> {
        Ok(self.stakes.read().await
            .values()
            .filter(|stake| stake.staker == *staker)
            .cloned()
            .collect())
    }

    /// Get total staked amount
    pub async fn get_total_staked(&self) -> u64 {
        *self.total_staked.read().await
    }

    /// Process expired stakes
    pub async fn process_expired_stakes(&self) -> SystemResult<()> {
        let mut stakes = self.stakes.write().await;
        let current_time = crate::utils::current_timestamp();

        for stake in stakes.values_mut() {
            if stake.status == StakeStatus::Active {
                let elapsed = current_time - stake.start_time;
                if elapsed >= stake.duration {
                    stake.status = StakeStatus::Unstaking {
                        unstake_time: current_time,
                    };
                    self.storage.put_stake(stake).await
                        .map_err(|e| SystemError::StakeError(e.to_string()))?;
                }
            }
        }

        Ok(())
    }
}