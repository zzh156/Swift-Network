use super::{AuthorityError, AuthorityResult, AuthorityStore};
use crate::protocol::{ProtocolError, ProtocolResult};
use crate::crypto::{KeyPair, PublicKey, Signature};
use crate::runtime::{Runtime, RuntimeConfig};
use crate::transaction::{Transaction, TransactionEffects, ExecutionStatus};
use crate::core::{Object, ObjectID};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Validator configuration
#[derive(Debug, Clone)]
pub struct ValidatorConfig {
    /// Runtime configuration
    pub runtime_config: RuntimeConfig,
    /// Maximum gas per transaction
    pub max_gas_per_tx: u64,
    /// Maximum concurrent transactions
    pub max_concurrent_txs: usize,
}

/// Validator state
#[derive(Debug)]
struct ValidatorState {
    /// Last executed sequence
    last_sequence: u64,
    /// Gas used in current epoch
    gas_used: u64,
    /// Transaction count in current epoch
    tx_count: u64,
}

/// Validator implementation
pub struct Validator {
    /// Configuration
    config: ValidatorConfig,
    /// Keypair for signing
    keypair: KeyPair,
    /// Authority store
    store: Arc<AuthorityStore>,
    /// Runtime
    runtime: Arc<Runtime>,
    /// State
    state: RwLock<ValidatorState>,
}

impl Validator {
    pub fn new(
        config: ValidatorConfig,
        keypair: KeyPair,
        store: Arc<AuthorityStore>,
    ) -> AuthorityResult<Self> {
        let runtime = Runtime::new(config.runtime_config.clone())
            .map_err(|e| AuthorityError::RuntimeError(e.to_string()))?;

        let state = ValidatorState {
            last_sequence: 0,
            gas_used: 0,
            tx_count: 0,
        };

        Ok(Self {
            config,
            keypair,
            store,
            runtime: Arc::new(runtime),
            state: RwLock::new(state),
        })
    }

    /// Execute transaction
    pub async fn execute_transaction(
        &self,
        transaction: Transaction,
    ) -> AuthorityResult<TransactionEffects> {
        // Check gas limit
        if transaction.gas_budget > self.config.max_gas_per_tx {
            return Err(AuthorityError::ExceedGasLimit);
        }

        // Create execution context
        let mut context = self.create_execution_context().await?;

        // Execute transaction
        let result = self.runtime.execute_transaction(
            transaction.clone(),
            &mut context,
        ).await.map_err(|e| AuthorityError::ExecutionError(e.to_string()))?;

        // Update state
        let mut state = self.state.write().await;
        state.last_sequence += 1;
        state.gas_used += result.gas_used;
        state.tx_count += 1;

        // Create effects
        let effects = TransactionEffects {
            transaction_digest: transaction.digest(),
            status: result.status,
            gas_used: result.gas_used,
            modified_objects: result.modified_objects,
            created_objects: result.created_objects,
            deleted_objects: result.deleted_objects,
            events: result.events,
            dependencies: transaction.dependencies,
            epoch_change: None,
        };

        // Store effects
        self.store.put_effects(&effects).await
            .map_err(|e| AuthorityError::StoreError(e.to_string()))?;

        Ok(effects)
    }

    /// Execute certificate
    pub async fn execute_certificate(
        &self,
        certificate: Certificate,
    ) -> AuthorityResult<TransactionEffects> {
        // Verify certificate first
        self.verify_certificate(&certificate).await?;

        // Execute transaction
        self.execute_transaction(certificate.transaction).await
    }

    /// Create execution context
    async fn create_execution_context(&self) -> AuthorityResult<ExecutionContext> {
        let state = self.state.read().await;
        
        Ok(ExecutionContext::new(
            state.last_sequence + 1,
            self.store.clone(),
            self.config.max_gas_per_tx,
        ))
    }

    /// Verify certificate
    async fn verify_certificate(&self, certificate: &Certificate) -> AuthorityResult<()> {
        // Verify epoch
        let state = self.state.read().await;
        if certificate.epoch != state.epoch {
            return Err(AuthorityError::InvalidEpoch {
                expected: state.epoch,
                actual: certificate.epoch,
            });
        }

        // Verify signatures
        let committee = self.get_committee().await?;
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

    /// Get committee info
    async fn get_committee(&self) -> AuthorityResult<CommitteeInfo> {
        self.store.get_committee().await
            .map_err(|e| AuthorityError::StoreError(e.to_string()))
    }

    /// Get validator metrics
    pub async fn get_metrics(&self) -> ValidatorMetrics {
        let state = self.state.read().await;
        ValidatorMetrics {
            last_sequence: state.last_sequence,
            gas_used: state.gas_used,
            tx_count: state.tx_count,
        }
    }
}

/// Validator metrics
#[derive(Debug, Clone)]
pub struct ValidatorMetrics {
    /// Last executed sequence
    pub last_sequence: u64,
    /// Gas used in current epoch
    pub gas_used: u64,
    /// Transaction count in current epoch
    pub tx_count: u64,
}

/// Execution context
pub struct ExecutionContext {
    /// Sequence number
    sequence: u64,
    /// Store
    store: Arc<AuthorityStore>,
    /// Gas limit
    gas_limit: u64,
    /// Modified objects
    modified_objects: Vec<Object>,
    /// Created objects
    created_objects: Vec<Object>,
    /// Deleted objects
    deleted_objects: Vec<ObjectID>,
    /// Events
    events: Vec<Event>,
}

impl ExecutionContext {
    pub fn new(
        sequence: u64,
        store: Arc<AuthorityStore>,
        gas_limit: u64,
    ) -> Self {
        Self {
            sequence,
            store,
            gas_limit,
            modified_objects: Vec::new(),
            created_objects: Vec::new(),
            deleted_objects: Vec::new(),
            events: Vec::new(),
        }
    }

    pub fn sequence(&self) -> u64 {
        self.sequence
    }

    pub fn gas_limit(&self) -> u64 {
        self.gas_limit
    }

    pub fn add_modified_object(&mut self, object: Object) {
        self.modified_objects.push(object);
    }

    pub fn add_created_object(&mut self, object: Object) {
        self.created_objects.push(object);
    }

    pub fn add_deleted_object(&mut self, id: ObjectID) {
        self.deleted_objects.push(id);
    }

    pub fn add_event(&mut self, event: Event) {
        self.events.push(event);
    }
}