use super::{DriverError, DriverResult, DriverStatus};
use crate::consensus::{ConsensusState, Certificate};
use crate::network::{NetworkService, NetworkMessage};
use crate::protocol::{Transaction, TransactionDigest, TransactionEffects};
use crate::storage::Storage;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, RwLock};
use tokio::time::timeout;

/// Driver configuration
#[derive(Debug, Clone)]
pub struct DriverConfig {
    /// Quorum size
    pub quorum_size: usize,
    /// Timeout duration
    pub timeout: Duration,
    /// Maximum pending transactions
    pub max_pending_transactions: usize,
    /// Maximum concurrent tasks
    pub max_concurrent_tasks: usize,
}

impl Default for DriverConfig {
    fn default() -> Self {
        Self {
            quorum_size: 2,
            timeout: Duration::from_secs(30),
            max_pending_transactions: 10000,
            max_concurrent_tasks: 100,
        }
    }
}

/// Pending transaction
struct PendingTransaction {
    /// Transaction
    transaction: Transaction,
    /// Collected signatures
    signatures: Vec<(PublicKey, Signature)>,
    /// Response sender
    response_sender: mpsc::Sender<DriverResult<TransactionEffects>>,
}

/// Quorum driver
pub struct QuorumDriver {
    /// Configuration
    config: DriverConfig,
    /// Network service
    network: Arc<NetworkService>,
    /// Storage
    storage: Arc<dyn Storage>,
    /// Driver status
    status: RwLock<DriverStatus>,
    /// Pending transactions
    pending_transactions: RwLock<HashMap<TransactionDigest, PendingTransaction>>,
    /// Transaction sender
    tx_sender: mpsc::Sender<(Transaction, mpsc::Sender<DriverResult<TransactionEffects>>)>,
}

impl QuorumDriver {
    /// Create new quorum driver
    pub fn new(
        config: DriverConfig,
        network: Arc<NetworkService>,
        storage: Arc<dyn Storage>,
    ) -> Self {
        let (tx_sender, tx_receiver) = mpsc::channel(config.max_pending_transactions);
        
        let driver = Self {
            config,
            network,
            storage,
            status: RwLock::new(DriverStatus::Active),
            pending_transactions: RwLock::new(HashMap::new()),
            tx_sender,
        };

        // Start transaction processor
        driver.start_transaction_processor(tx_receiver);

        driver
    }

    /// Start transaction processor
    fn start_transaction_processor(
        &self,
        mut tx_receiver: mpsc::Receiver<(Transaction, mpsc::Sender<DriverResult<TransactionEffects>>)>,
    ) {
        let driver = Arc::new(self.clone());
        
        tokio::spawn(async move {
            while let Some((transaction, response_sender)) = tx_receiver.recv().await {
                let driver = driver.clone();
                
                tokio::spawn(async move {
                    let result = driver.process_transaction(transaction, response_sender.clone()).await;
                    if let Err(e) = result {
                        let _ = response_sender.send(Err(e)).await;
                    }
                });
            }
        });
    }

    /// Process transaction
    async fn process_transaction(
        &self,
        transaction: Transaction,
        response_sender: mpsc::Sender<DriverResult<TransactionEffects>>,
    ) -> DriverResult<()> {
        // Check driver status
        if *self.status.read().await != DriverStatus::Active {
            return Err(DriverError::ConsensusError("Driver not active".into()));
        }

        // Get transaction digest
        let digest = transaction.digest();

        // Create pending transaction
        let pending = PendingTransaction {
            transaction: transaction.clone(),
            signatures: Vec::new(),
            response_sender,
        };

        // Add to pending transactions
        self.pending_transactions.write().await.insert(digest, pending);

        // Broadcast transaction
        self.network.broadcast(NetworkMessage::Transaction(transaction)).await
            .map_err(|e| DriverError::NetworkError(e.to_string()))?;

        // Wait for quorum with timeout
        if let Err(_) = timeout(self.config.timeout, self.wait_for_quorum(digest)).await {
            // Remove pending transaction
            self.pending_transactions.write().await.remove(&digest);
            return Err(DriverError::TimeoutError("Quorum timeout".into()));
        }

        Ok(())
    }

    /// Wait for quorum
    async fn wait_for_quorum(&self, digest: TransactionDigest) -> DriverResult<()> {
        loop {
            // Get pending transaction
            let pending = self.pending_transactions.read().await
                .get(&digest)
                .ok_or_else(|| DriverError::ConsensusError("Transaction not found".into()))?
                .clone();

            // Check if we have quorum
            if pending.signatures.len() >= self.config.quorum_size {
                // Create certificate
                let certificate = Certificate::new(
                    pending.transaction,
                    pending.signatures,
                );

                // Execute certificate
                let effects = self.execute_certificate(certificate).await?;

                // Send response
                pending.response_sender.send(Ok(effects)).await
                    .map_err(|e| DriverError::ConsensusError(e.to_string()))?;

                // Remove pending transaction
                self.pending_transactions.write().await.remove(&digest);

                return Ok(());
            }

            // Wait a bit before checking again
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }

    /// Execute certificate
    async fn execute_certificate(&self, certificate: Certificate) -> DriverResult<TransactionEffects> {
        // Verify certificate
        certificate.verify()
            .map_err(|e| DriverError::CertificateError(e.to_string()))?;

        // Store certificate
        self.storage.put_certificate(&certificate).await
            .map_err(|e| DriverError::ConsensusError(e.to_string()))?;

        // Execute transaction
        let effects = self.storage.execute_certificate(&certificate).await
            .map_err(|e| DriverError::ConsensusError(e.to_string()))?;

        Ok(effects)
    }

    /// Submit transaction
    pub async fn submit_transaction(
        &self,
        transaction: Transaction,
    ) -> DriverResult<TransactionEffects> {
        // Create response channel
        let (response_sender, mut response_receiver) = mpsc::channel(1);

        // Send transaction to processor
        self.tx_sender.send((transaction, response_sender)).await
            .map_err(|e| DriverError::ConsensusError(e.to_string()))?;

        // Wait for response
        response_receiver.recv().await
            .ok_or_else(|| DriverError::ConsensusError("Response channel closed".into()))?
    }

    /// Handle signature
    pub async fn handle_signature(
        &self,
        digest: TransactionDigest,
        public_key: PublicKey,
        signature: Signature,
    ) -> DriverResult<()> {
        // Get pending transaction
        let mut pending_transactions = self.pending_transactions.write().await;
        let pending = pending_transactions.get_mut(&digest)
            .ok_or_else(|| DriverError::ConsensusError("Transaction not found".into()))?;

        // Add signature
        pending.signatures.push((public_key, signature));

        Ok(())
    }

    /// Get driver status
    pub async fn status(&self) -> DriverStatus {
        self.status.read().await.clone()
    }

    /// Set driver status
    pub async fn set_status(&self, status: DriverStatus) {
        *self.status.write().await = status;
    }
}