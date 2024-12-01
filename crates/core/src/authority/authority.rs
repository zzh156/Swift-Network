use super::{
    AuthorityError, AuthorityResult, AuthorityState, CommitteeInfo,
    AuthorityStore, CheckpointStore, EpochManager, Validator,
};
use crate::protocol::{ProtocolError, ProtocolResult};
use crate::crypto::{KeyPair, PublicKey, Signature};
use crate::core::{Object, ObjectID};
use crate::transaction::{Transaction, TransactionDigest, TransactionEffects};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Authority configuration
#[derive(Debug, Clone)]
pub struct AuthorityConfig {
    /// Keypair for signing
    pub keypair: KeyPair,
    /// Network address
    pub network_address: String,
    /// Store configuration
    pub store_config: StoreConfig,
    /// Initial stake
    pub initial_stake: u64,
}

/// Authority implementation
pub struct Authority {
    /// Configuration
    config: AuthorityConfig,
    /// Authority store
    store: Arc<AuthorityStore>,
    /// Checkpoint store
    checkpoint_store: Arc<CheckpointStore>,
    /// Epoch manager
    epoch_manager: Arc<EpochManager>,
    /// Validator
    validator: Arc<Validator>,
    /// Current state
    state: RwLock<AuthorityState>,
}

impl Authority {
    pub fn new(config: AuthorityConfig) -> AuthorityResult<Self> {
        let store = Arc::new(AuthorityStore::new(config.store_config.clone())?);
        let checkpoint_store = Arc::new(CheckpointStore::new(store.clone())?);
        let epoch_manager = Arc::new(EpochManager::new(store.clone())?);
        
        let validator = Arc::new(Validator::new(
            config.keypair.clone(),
            store.clone(),
        )?);

        let state = RwLock::new(AuthorityState {
            public_key: config.keypair.public(),
            epoch: 0,
            stake: config.initial_stake,
            network_address: config.network_address.clone(),
        });

        Ok(Self {
            config,
            store,
            checkpoint_store,
            epoch_manager,
            validator,
            state,
        })
    }

    /// Handle transaction
    pub async fn handle_transaction(
        &self,
        transaction: Transaction,
    ) -> AuthorityResult<TransactionEffects> {
        // Verify transaction
        self.verify_transaction(&transaction).await?;

        // Execute transaction
        let effects = self.validator.execute_transaction(transaction).await?;

        // Update state if needed
        if effects.epoch_change.is_some() {
            self.update_epoch(effects.epoch_change.as_ref().unwrap()).await?;
        }

        Ok(effects)
    }

    /// Handle certificate
    pub async fn handle_certificate(
        &self,
        certificate: Certificate,
    ) -> AuthorityResult<TransactionEffects> {
        // Verify certificate
        self.verify_certificate(&certificate).await?;

        // Execute certificate
        let effects = self.validator.execute_certificate(certificate).await?;

        // Update state if needed
        if effects.epoch_change.is_some() {
            self.update_epoch(effects.epoch_change.as_ref().unwrap()).await?;
        }

        Ok(effects)
    }

    /// Sign transaction
    pub async fn sign_transaction(
        &self,
        transaction: &Transaction,
    ) -> AuthorityResult<Signature> {
        // Verify transaction first
        self.verify_transaction(transaction).await?;

        // Sign transaction
        let signature = self.config.keypair.sign(transaction.digest().as_ref());
        Ok(signature)
    }

    /// Verify transaction
    async fn verify_transaction(&self, transaction: &Transaction) -> AuthorityResult<()> {
        // Verify epoch
        let state = self.state.read().await;
        if transaction.epoch != state.epoch {
            return Err(AuthorityError::InvalidEpoch {
                expected: state.epoch,
                actual: transaction.epoch,
            });
        }

        // Verify signature
        if !transaction.verify_signature() {
            return Err(AuthorityError::InvalidSignature);
        }

        Ok(())
    }

    /// Verify certificate
    async fn verify_certificate(&self, certificate: &Certificate) -> AuthorityResult<()> {
        // Verify transaction
        self.verify_transaction(&certificate.transaction).await?;

        // Verify signatures
        let committee = self.epoch_manager.get_committee().await?;
        let mut total_stake = 0;

        for (public_key, signature) in &certificate.signatures {
            // Verify signature
            if !signature.verify(
                certificate.transaction.digest().as_ref(),
                public_key,
            ) {
                return Err(AuthorityError::InvalidSignature);
            }

            // Add stake
            if let Some(validator) = committee.get_validator(public_key) {
                total_stake += validator.stake;
            }
        }

        // Verify quorum
        if !committee.has_quorum(total_stake) {
            return Err(AuthorityError::InvalidStake(
                "Insufficient stake for quorum".into()
            ));
        }

        Ok(())
    }

    /// Update epoch
    async fn update_epoch(&self, new_epoch: &EpochInfo) -> AuthorityResult<()> {
        let mut state = self.state.write().await;
        state.epoch = new_epoch.epoch;
        state.stake = new_epoch.get_stake(&state.public_key)
            .ok_or_else(|| AuthorityError::InvalidStake("Validator not in committee".into()))?;

        Ok(())
    }

    /// Get object
    pub async fn get_object(&self, id: &ObjectID) -> AuthorityResult<Option<Object>> {
        self.store.get_object(id).await.map_err(|e| {
            AuthorityError::StoreError(e.to_string())
        })
    }

    /// Get transaction
    pub async fn get_transaction(
        &self,
        digest: &TransactionDigest,
    ) -> AuthorityResult<Option<Transaction>> {
        self.store.get_transaction(digest).await.map_err(|e| {
            AuthorityError::StoreError(e.to_string())
        })
    }

    /// Get transaction effects
    pub async fn get_transaction_effects(
        &self,
        digest: &TransactionDigest,
    ) -> AuthorityResult<Option<TransactionEffects>> {
        self.store.get_effects(digest).await.map_err(|e| {
            AuthorityError::StoreError(e.to_string())
        })
    }

    /// Get checkpoint
    pub async fn get_checkpoint(
        &self,
        sequence: u64,
    ) -> AuthorityResult<Option<Checkpoint>> {
        self.checkpoint_store.get_checkpoint(sequence).await.map_err(|e| {
            AuthorityError::CheckpointError(e.to_string())
        })
    }
}