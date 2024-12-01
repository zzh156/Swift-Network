use super::{SystemError, SystemResult};
use crate::core::{Address, ObjectID};
use crate::storage::Storage;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Reward configuration
#[derive(Debug, Clone)]
pub struct RewardConfig {
    /// Base reward rate (per epoch)
    pub base_reward_rate: f64,
    /// Minimum stake for rewards
    pub min_stake_for_rewards: u64,
    /// Maximum reward per epoch
    pub max_reward_per_epoch: u64,
    /// Reward distribution interval
    pub distribution_interval: u64,
}

/// Reward type
#[derive(Debug, Clone)]
pub enum RewardType {
    /// Staking reward
    Staking {
        /// Stake amount
        stake_amount: u64,
        /// Stake duration
        stake_duration: u64,
    },
    /// Validator reward
    Validator {
        /// Blocks proposed
        blocks_proposed: u64,
        /// Transactions processed
        transactions_processed: u64,
    },
    /// Governance reward
    Governance {
        /// Proposals created
        proposals_created: u64,
        /// Votes cast
        votes_cast: u64,
    },
}

/// Reward distribution
#[derive(Debug, Clone)]
pub struct RewardDistribution {
    /// Epoch number
    pub epoch: u64,
    /// Timestamp
    pub timestamp: u64,
    /// Total reward
    pub total_reward: u64,
    /// Distributions
    pub distributions: Vec<(Address, u64)>,
}

/// Reward system
pub struct RewardSystem {
    /// Configuration
    config: RewardConfig,
    /// Storage
    storage: Arc<dyn Storage>,
    /// Current epoch
    current_epoch: RwLock<u64>,
    /// Pending rewards
    pending_rewards: RwLock<HashMap<Address, Vec<(RewardType, u64)>>>,
}

impl RewardSystem {
    /// Create new reward system
    pub fn new(
        config: RewardConfig,
        storage: Arc<dyn Storage>,
    ) -> Self {
        Self {
            config,
            storage,
            current_epoch: RwLock::new(0),
            pending_rewards: RwLock::new(HashMap::new()),
        }
    }

    /// Initialize reward system
    pub async fn initialize(&mut self) -> SystemResult<()> {
        // Load current epoch
        let epoch = self.storage.get_current_epoch().await
            .map_err(|e| SystemError::RewardError(e.to_string()))?;
        *self.current_epoch.write().await = epoch;

        // Load pending rewards
        let pending_rewards = self.storage.get_pending_rewards().await
            .map_err(|e| SystemError::RewardError(e.to_string()))?;
        *self.pending_rewards.write().await = pending_rewards;

        Ok(())
    }

    /// Add reward
    pub async fn add_reward(
        &self,
        address: Address,
        reward_type: RewardType,
    ) -> SystemResult<()> {
        // Calculate reward amount
        let amount = self.calculate_reward_amount(&reward_type)?;

        // Add to pending rewards
        let mut pending_rewards = self.pending_rewards.write().await;
        pending_rewards.entry(address)
            .or_insert_with(Vec::new)
            .push((reward_type, amount));

        // Store updated pending rewards
        self.storage.put_pending_rewards(&*pending_rewards).await
            .map_err(|e| SystemError::RewardError(e.to_string()))?;

        Ok(())
    }

    /// Calculate reward amount
    fn calculate_reward_amount(&self, reward_type: &RewardType) -> SystemResult<u64> {
        let amount = match reward_type {
            RewardType::Staking { stake_amount, stake_duration } => {
                if *stake_amount < self.config.min_stake_for_rewards {
                    return Err(SystemError::RewardError("Insufficient stake".into()));
                }
                let base = (*stake_amount as f64 * self.config.base_reward_rate) as u64;
                let duration_bonus = (*stake_duration as f64 * 0.1) as u64;
                base + duration_bonus
            }
            RewardType::Validator { blocks_proposed, transactions_processed } => {
                let block_reward = *blocks_proposed * 100;
                let tx_reward = *transactions_processed * 1;
                block_reward + tx_reward
            }
            RewardType::Governance { proposals_created, votes_cast } => {
                let proposal_reward = *proposals_created * 1000;
                let vote_reward = *votes_cast * 10;
                proposal_reward + vote_reward
            }
        };

        Ok(amount.min(self.config.max_reward_per_epoch))
    }

    /// Distribute rewards
    pub async fn distribute_rewards(&self) -> SystemResult<RewardDistribution> {
        // Check distribution interval
        let current_epoch = *self.current_epoch.read().await;
        if current_epoch % self.config.distribution_interval != 0 {
            return Err(SystemError::RewardError("Not distribution epoch".into()));
        }

        // Get pending rewards
        let pending_rewards = self.pending_rewards.read().await;
        if pending_rewards.is_empty() {
            return Err(SystemError::RewardError("No pending rewards".into()));
        }

        // Calculate distributions
        let mut distributions = Vec::new();
        let mut total_reward = 0;

        for (address, rewards) in pending_rewards.iter() {
            let reward_sum: u64 = rewards.iter()
                .map(|(_, amount)| amount)
                .sum();
            distributions.push((*address, reward_sum));
            total_reward += reward_sum;
        }

        // Create distribution
        let distribution = RewardDistribution {
            epoch: current_epoch,
            timestamp: crate::utils::current_timestamp(),
            total_reward,
            distributions,
        };

        // Store distribution
        self.storage.put_reward_distribution(&distribution).await
            .map_err(|e| SystemError::RewardError(e.to_string()))?;

        // Clear pending rewards
        self.pending_rewards.write().await.clear();
        self.storage.clear_pending_rewards().await
            .map_err(|e| SystemError::RewardError(e.to_string()))?;

        Ok(distribution)
    }

    /// Get pending rewards
    pub async fn get_pending_rewards(&self, address: &Address) -> SystemResult<Vec<(RewardType, u64)>> {
        Ok(self.pending_rewards.read().await
            .get(address)
            .cloned()
            .unwrap_or_default())
    }

    /// Get reward distributions
    pub async fn get_reward_distributions(
        &self,
        start_epoch: u64,
        end_epoch: u64,
    ) -> SystemResult<Vec<RewardDistribution>> {
        self.storage.get_reward_distributions(start_epoch, end_epoch).await
            .map_err(|e| SystemError::RewardError(e.to_string()))
    }
}