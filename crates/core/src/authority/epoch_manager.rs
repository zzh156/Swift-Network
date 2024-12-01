use super::{AuthorityError, AuthorityResult, AuthorityStore, CommitteeInfo};
use crate::crypto::PublicKey;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Epoch configuration
#[derive(Debug, Clone)]
pub struct EpochConfig {
    /// Epoch duration in milliseconds
    pub epoch_duration_ms: u64,
    /// Minimum validator stake
    pub min_validator_stake: u64,
    /// Maximum validator count
    pub max_validator_count: usize,
}

/// Epoch information
#[derive(Debug, Clone)]
pub struct EpochInfo {
    /// Epoch number
    pub epoch: u64,
    /// Start timestamp
    pub start_timestamp: u64,
    /// End timestamp
    pub end_timestamp: u64,
    /// Committee information
    pub committee: CommitteeInfo,
    /// Total transactions
    pub total_transactions: u64,
    /// Total gas used
    pub total_gas_used: u64,
}

impl EpochInfo {
    /// Create genesis epoch
    pub fn genesis() -> Self {
        let committee = CommitteeInfo {
            epoch: 0,
            validators: Vec::new(),
            quorum_threshold: 0,
            total_stake: 0,
        };

        Self {
            epoch: 0,
            start_timestamp: 0,
            end_timestamp: 0,
            committee,
            total_transactions: 0,
            total_gas_used: 0,
        }
    }

    /// Get validator stake
    pub fn get_stake(&self, public_key: &PublicKey) -> Option<u64> {
        self.committee.validators.iter()
            .find(|v| v.public_key == *public_key)
            .map(|v| v.stake)
    }

    /// Check if epoch has ended
    pub fn has_ended(&self, current_timestamp: u64) -> bool {
        current_timestamp >= self.end_timestamp
    }
}

/// Epoch manager
pub struct EpochManager {
    /// Configuration
    config: EpochConfig,
    /// Authority store
    store: Arc<AuthorityStore>,
    /// Current epoch
    current_epoch: RwLock<EpochInfo>,
    /// Next epoch committee
    next_committee: RwLock<Option<CommitteeInfo>>,
}

impl EpochManager {
    pub fn new(
        config: EpochConfig,
        store: Arc<AuthorityStore>,
    ) -> AuthorityResult<Self> {
        // Load current epoch
        let current_epoch = store.get_current_epoch()
            .map_err(|e| AuthorityError::StoreError(e.to_string()))?
            .unwrap_or_else(EpochInfo::genesis);

        Ok(Self {
            config,
            store,
            current_epoch: RwLock::new(current_epoch),
            next_committee: RwLock::new(None),
        })
    }

    /// Get current epoch
    pub async fn get_current_epoch(&self) -> EpochInfo {
        self.current_epoch.read().await.clone()
    }

    /// Get next committee
    pub async fn get_next_committee(&self) -> Option<CommitteeInfo> {
        self.next_committee.read().await.clone()
    }

    /// Prepare next epoch
    pub async fn prepare_next_epoch(
        &self,
        validators: Vec<(PublicKey, u64)>,
    ) -> AuthorityResult<CommitteeInfo> {
        // Validate validators
        if validators.len() > self.config.max_validator_count {
            return Err(AuthorityError::TooManyValidators);
        }

        let mut committee_validators = Vec::new();
        let mut total_stake = 0;

        for (public_key, stake) in validators {
            if stake < self.config.min_validator_stake {
                return Err(AuthorityError::InsufficientValidatorStake);
            }

            committee_validators.push(AuthorityState {
                public_key,
                epoch: self.current_epoch.read().await.epoch + 1,
                stake,
                network_address: String::new(), // Will be set later
            });

            total_stake += stake;
        }

        let committee = CommitteeInfo {
            epoch: self.current_epoch.read().await.epoch + 1,
            validators: committee_validators,
            quorum_threshold: (total_stake * 2) / 3 + 1,
            total_stake,
        };

        *self.next_committee.write().await = Some(committee.clone());

        Ok(committee)
    }

    /// Start new epoch
    pub async fn start_new_epoch(&self, timestamp: u64) -> AuthorityResult<EpochInfo> {
        let mut current = self.current_epoch.write().await;
        let next_committee = self.next_committee.write().await.take()
            .ok_or(AuthorityError::NoNextEpochCommittee)?;

        let new_epoch = EpochInfo {
            epoch: current.epoch + 1,
            start_timestamp: timestamp,
            end_timestamp: timestamp + self.config.epoch_duration_ms,
            committee: next_committee,
            total_transactions: 0,
            total_gas_used: 0,
        };

        // Store new epoch
        self.store.put_current_epoch(&new_epoch)
            .map_err(|e| AuthorityError::StoreError(e.to_string()))?;

        // Update current epoch
        *current = new_epoch.clone();

        Ok(new_epoch)
    }

    /// Update epoch statistics
    pub async fn update_statistics(
        &self,
        gas_used: u64,
    ) -> AuthorityResult<()> {
        let mut current = self.current_epoch.write().await;
        
        current.total_transactions += 1;
        current.total_gas_used += gas_used;

        // Store updated epoch
        self.store.put_current_epoch(&current)
            .map_err(|e| AuthorityError::StoreError(e.to_string()))?;

        Ok(())
    }

    /// Check if current epoch should end
    pub async fn should_end_epoch(&self, timestamp: u64) -> bool {
        let current = self.current_epoch.read().await;
        current.has_ended(timestamp)
    }

    /// Get epoch by number
    pub async fn get_epoch(&self, epoch: u64) -> AuthorityResult<Option<EpochInfo>> {
        self.store.get_epoch(epoch)
            .map_err(|e| AuthorityError::StoreError(e.to_string()))
    }
}