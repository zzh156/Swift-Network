use super::{SystemError, SystemResult};
use crate::core::{Address, ObjectID};
use crate::crypto::PublicKey;
use crate::storage::Storage;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Validator configuration
#[derive(Debug, Clone)]
pub struct ValidatorConfig {
    /// Minimum stake amount
    pub min_stake_amount: u64,
    /// Maximum validator count
    pub max_validator_count: usize,
    /// Performance tracking window
    pub performance_window: u64,
    /// Minimum performance threshold
    pub min_performance_threshold: f64,
}

/// Validator info
#[derive(Debug, Clone)]
pub struct ValidatorInfo {
    /// Validator ID
    pub id: ObjectID,
    /// Public key
    pub public_key: PublicKey,
    /// Network address
    pub network_address: String,
    /// Stake amount
    pub stake_amount: u64,
    /// Commission rate
    pub commission_rate: f64,
    /// Performance metrics
    pub performance: ValidatorPerformance,
    /// Status
    pub status: ValidatorStatus,
}

/// Validator performance
#[derive(Debug, Clone)]
pub struct ValidatorPerformance {
    /// Blocks proposed
    pub blocks_proposed: u64,
    /// Blocks signed
    pub blocks_signed: u64,
    /// Response time (ms)
    pub response_time: u64,
    /// Uptime percentage
    pub uptime: f64,
}

/// Validator status
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidatorStatus {
    /// Active
    Active,
    /// Jailed
    Jailed {
        /// Jail time
        jail_time: u64,
        /// Reason
        reason: String,
    },
    /// Inactive
    Inactive,
}

/// Validator set
pub struct ValidatorSet {
    /// Configuration
    config: ValidatorConfig,
    /// Storage
    storage: Arc<dyn Storage>,
    /// Validators
    validators: RwLock<HashMap<ObjectID, ValidatorInfo>>,
    /// Active set
    active_set: RwLock<Vec<ObjectID>>,
}

impl ValidatorSet {
    /// Create new validator set
    pub fn new(
        config: ValidatorConfig,
        storage: Arc<dyn Storage>,
    ) -> Self {
        Self {
            config,
            storage,
            validators: RwLock::new(HashMap::new()),
            active_set: RwLock::new(Vec::new()),
        }
    }

    /// Initialize validator set
    pub async fn initialize(&mut self) -> SystemResult<()> {
        // Load validators
        let validators = self.storage.get_validators().await
            .map_err(|e| SystemError::ValidatorError(e.to_string()))?;
        *self.validators.write().await = validators;

        // Build active set
        let active_set: Vec<_> = validators.iter()
            .filter(|(_, v)| v.status == ValidatorStatus::Active)
            .map(|(id, _)| *id)
            .collect();
        *self.active_set.write().await = active_set;

        Ok(())
    }

    /// Register validator
    pub async fn register_validator(
        &self,
        public_key: PublicKey,
        network_address: String,
        stake_amount: u64,
        commission_rate: f64,
    ) -> SystemResult<ObjectID> {
        // Validate stake amount
        if stake_amount < self.config.min_stake_amount {
            return Err(SystemError::ValidatorError("Insufficient stake".into()));
        }

        // Check validator count
        if self.validators.read().await.len() >= self.config.max_validator_count {
            return Err(SystemError::ValidatorError("Maximum validator count reached".into()));
        }

        // Create validator info
        let validator = ValidatorInfo {
            id: ObjectID::random(),
            public_key,
            network_address,
            stake_amount,
            commission_rate,
            performance: ValidatorPerformance {
                blocks_proposed: 0,
                blocks_signed: 0,
                response_time: 0,
                uptime: 100.0,
            },
            status: ValidatorStatus::Active,
        };

        // Store validator
        self.storage.put_validator(&validator).await
            .map_err(|e| SystemError::ValidatorError(e.to_string()))?;
        self.validators.write().await.insert(validator.id, validator.clone());

        // Update active set
        self.active_set.write().await.push(validator.id);

        Ok(validator.id)
    }

    /// Update validator stake
    pub async fn update_stake(
        &self,
        validator_id: ObjectID,
        new_stake: u64,
    ) -> SystemResult<()> {
        // Get validator
        let mut validators = self.validators.write().await;
        let validator = validators.get_mut(&validator_id)
            .ok_or_else(|| SystemError::ValidatorError("Validator not found".into()))?;

        // Validate stake amount
        if new_stake < self.config.min_stake_amount {
            return Err(SystemError::ValidatorError("Insufficient stake".into()));
        }

        // Update stake
        validator.stake_amount = new_stake;

        // Store updated validator
        self.storage.put_validator(validator).await
            .map_err(|e| SystemError::ValidatorError(e.to_string()))?;

        Ok(())
    }

    /// Update validator performance
    pub async fn update_performance(
        &self,
        validator_id: ObjectID,
        blocks_proposed: u64,
        blocks_signed: u64,
        response_time: u64,
        uptime: f64,
    ) -> SystemResult<()> {
        // Get validator
        let mut validators = self.validators.write().await;
        let validator = validators.get_mut(&validator_id)
            .ok_or_else(|| SystemError::ValidatorError("Validator not found".into()))?;

        // Update performance
        validator.performance = ValidatorPerformance {
            blocks_proposed,
            blocks_signed,
            response_time,
            uptime,
        };

        // Check performance threshold
        if uptime < self.config.min_performance_threshold {
            validator.status = ValidatorStatus::Jailed {
                jail_time: crate::utils::current_timestamp(),
                reason: "Poor performance".into(),
            };
            self.active_set.write().await.retain(|id| *id != validator_id);
        }

        // Store updated validator
        self.storage.put_validator(validator).await
            .map_err(|e| SystemError::ValidatorError(e.to_string()))?;

        Ok(())
    }

    /// Jail validator
    pub async fn jail_validator(
        &self,
        validator_id: ObjectID,
        reason: String,
    ) -> SystemResult<()> {
        // Get validator
        let mut validators = self.validators.write().await;
        let validator = validators.get_mut(&validator_id)
            .ok_or_else(|| SystemError::ValidatorError("Validator not found".into()))?;

        // Update status
        validator.status = ValidatorStatus::Jailed {
            jail_time: crate::utils::current_timestamp(),
            reason,
        };

        // Remove from active set
        self.active_set.write().await.retain(|id| *id != validator_id);

        // Store updated validator
        self.storage.put_validator(validator).await
            .map_err(|e| SystemError::ValidatorError(e.to_string()))?;

        Ok(())
    }

    /// Unjail validator
    pub async fn unjail_validator(&self, validator_id: ObjectID) -> SystemResult<()> {
        // Get validator
        let mut validators = self.validators.write().await;
        let validator = validators.get_mut(&validator_id)
            .ok_or_else(|| SystemError::ValidatorError("Validator not found".into()))?;

        // Check status
        match validator.status {
            ValidatorStatus::Jailed { .. } => {
                validator.status = ValidatorStatus::Active;
                self.active_set.write().await.push(validator_id);
            }
            _ => return Err(SystemError::ValidatorError("Validator not jailed".into())),
        }

        // Store updated validator
        self.storage.put_validator(validator).await
            .map_err(|e| SystemError::ValidatorError(e.to_string()))?;

        Ok(())
    }

    /// Get validator info
    pub async fn get_validator(&self, validator_id: &ObjectID) -> SystemResult<Option<ValidatorInfo>> {
        Ok(self.validators.read().await.get(validator_id).cloned())
    }

    /// Get active validators
    pub async fn get_active_validators(&self) -> SystemResult<Vec<ValidatorInfo>> {
        let active_set = self.active_set.read().await;
        let validators = self.validators.read().await;
        
        Ok(active_set.iter()
            .filter_map(|id| validators.get(id))
            .cloned()
            .collect())
    }

    /// Get total stake
    pub async fn get_total_stake(&self) -> u64 {
        self.validators.read().await
            .values()
            .filter(|v| v.status == ValidatorStatus::Active)
            .map(|v| v.stake_amount)
            .sum()
    }
}