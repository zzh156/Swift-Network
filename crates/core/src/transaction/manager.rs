use super::{Transaction, TransactionDigest, ValidationResult};
use crate::core::ObjectID;
use crate::execution::{ExecutionEffects, Executor};
use crate::storage::Storage;
use crate::protocol::{ProtocolError, ProtocolResult};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Transaction information
#[derive(Debug, Clone)]
pub struct TransactionInfo {
    /// Transaction
    pub transaction: Transaction,
    /// Transaction status
    pub status: TransactionStatus,
    /// Execution effects
    pub effects: Option<ExecutionEffects>,
    /// Creation timestamp
    pub timestamp: u64,
}

/// Transaction status
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransactionStatus {
    /// Pending in mempool
    Pending,
    /// Being processed
    Processing,
    /// Executed successfully
    Executed,
    /// Failed execution
    Failed(String),
}

/// Transaction manager
pub struct TransactionManager {
    /// Storage
    storage: Arc<dyn Storage>,
    /// Executor
    executor: Arc<Executor>,
    /// Transaction validator
    validator: Arc<TransactionValidator>,
    /// Processing transactions
    processing: RwLock<HashMap<TransactionDigest, TransactionInfo>>,
}

impl TransactionManager {
    /// Create new transaction manager
    pub fn new(
        storage: Arc<dyn Storage>,
        executor: Arc<Executor>,
        validator: Arc<TransactionValidator>,
    ) -> Self {
        Self {
            storage,
            executor,
            validator,
            processing: RwLock::new(HashMap::new()),
        }
    }

    /// Submit transaction
    pub async fn submit_transaction(
        &self,
        transaction: Transaction,
    ) -> ProtocolResult<TransactionDigest> {
        // Validate transaction
        self.validator.validate_transaction(&transaction)?;

        // Get transaction digest
        let digest = transaction.digest();

        // Check if already exists
        if self.storage.get_transaction(&digest).await?.is_some() {
            return Err(ProtocolError::TransactionExists(digest));
        }

        // Create transaction info
        let info = TransactionInfo {
            transaction: transaction.clone(),
            status: TransactionStatus::Pending,
            effects: None,
            timestamp: crate::utils::current_timestamp(),
        };

        // Store transaction
        self.storage.put_transaction(&digest, &info).await?;

        // Add to processing queue
        self.processing.write().await.insert(digest, info);

        Ok(digest)
    }

    /// Execute transaction
    pub async fn execute_transaction(
        &self,
        digest: &TransactionDigest,
    ) -> ProtocolResult<ExecutionEffects> {
        // Get transaction info
        let mut info = self.get_transaction_info(digest).await?;

        // Check status
        if info.status != TransactionStatus::Pending {
            return Err(ProtocolError::InvalidTransactionStatus);
        }

        // Update status
        info.status = TransactionStatus::Processing;
        self.update_transaction_info(&info).await?;

        // Execute transaction
        let effects = match self.executor.execute_transaction(&info.transaction).await {
            Ok(effects) => {
                info.status = TransactionStatus::Executed;
                info.effects = Some(effects.clone());
                effects
            }
            Err(e) => {
                info.status = TransactionStatus::Failed(e.to_string());
                return Err(ProtocolError::ExecutionError(e));
            }
        };

        // Update transaction info
        self.update_transaction_info(&info).await?;

        // Remove from processing queue
        self.processing.write().await.remove(digest);

        Ok(effects)
    }

    /// Get transaction info
    pub async fn get_transaction_info(
        &self,
        digest: &TransactionDigest,
    ) -> ProtocolResult<TransactionInfo> {
        // Check processing queue first
        if let Some(info) = self.processing.read().await.get(digest) {
            return Ok(info.clone());
        }

        // Get from storage
        self.storage.get_transaction(digest).await?
            .ok_or_else(|| ProtocolError::TransactionNotFound(*digest))
    }

    /// Update transaction info
    async fn update_transaction_info(&self, info: &TransactionInfo) -> ProtocolResult<()> {
        // Update storage
        self.storage.put_transaction(&info.transaction.digest(), info).await?;

        // Update processing queue
        if let Some(processing_info) = self.processing.write().await.get_mut(&info.transaction.digest()) {
            *processing_info = info.clone();
        }

        Ok(())
    }

    /// Get transaction effects
    pub async fn get_transaction_effects(
        &self,
        digest: &TransactionDigest,
    ) -> ProtocolResult<Option<ExecutionEffects>> {
        Ok(self.get_transaction_info(digest).await?.effects)
    }

    /// Get processing transactions
    pub async fn get_processing_transactions(&self) -> Vec<TransactionInfo> {
        self.processing.read().await.values().cloned().collect()
    }
}