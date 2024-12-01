use super::{MempoolError, MempoolResult, Priority, TransactionPrioritizer};
use crate::protocol::{SignedTransaction, TransactionDigest};
use std::collections::{HashMap, BTreeMap};
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Mempool configuration
#[derive(Debug, Clone)]
pub struct MempoolConfig {
    /// Maximum number of transactions
    pub capacity: usize,
    /// Transaction timeout
    pub transaction_timeout: Duration,
    /// Maximum size per account
    pub per_account_limit: usize,
}

impl Default for MempoolConfig {
    fn default() -> Self {
        Self {
            capacity: 10_000,
            transaction_timeout: Duration::from_secs(30),
            per_account_limit: 100,
        }
    }
}

/// Main mempool structure
pub struct Mempool {
    /// Configuration
    config: MempoolConfig,
    /// Transaction storage
    transactions: RwLock<HashMap<TransactionDigest, TransactionInfo>>,
    /// Priority queue
    priority_queue: RwLock<BTreeMap<Priority, Vec<TransactionDigest>>>,
    /// Per-account transaction count
    account_txs: RwLock<HashMap<String, usize>>,
    /// Transaction prioritizer
    prioritizer: TransactionPrioritizer,
}

/// Transaction information
#[derive(Debug, Clone)]
struct TransactionInfo {
    transaction: SignedTransaction,
    priority: Priority,
    insertion_time: Instant,
}

impl Mempool {
    /// Create new mempool
    pub fn new(config: MempoolConfig) -> Self {
        Self {
            config,
            transactions: RwLock::new(HashMap::new()),
            priority_queue: RwLock::new(BTreeMap::new()),
            account_txs: RwLock::new(HashMap::new()),
            prioritizer: TransactionPrioritizer::new(),
        }
    }

    /// Add transaction to mempool
    pub async fn add_transaction(
        &self,
        transaction: SignedTransaction,
    ) -> MempoolResult<()> {
        // Check capacity
        if self.transactions.read().await.len() >= self.config.capacity {
            return Err(MempoolError::MempoolFull);
        }

        let digest = transaction.digest();
        let sender = transaction.sender().to_string();

        // Check duplicates
        if self.transactions.read().await.contains_key(&digest) {
            return Err(MempoolError::DuplicateTransaction);
        }

        // Check per-account limit
        let mut account_txs = self.account_txs.write().await;
        let count = account_txs.entry(sender.clone()).or_insert(0);
        if *count >= self.config.per_account_limit {
            return Err(MempoolError::MempoolFull);
        }

        // Calculate priority
        let priority = self.prioritizer.calculate_priority(&transaction);

        // Add transaction
        let info = TransactionInfo {
            transaction,
            priority,
            insertion_time: Instant::now(),
        };

        self.transactions.write().await.insert(digest, info.clone());
        self.priority_queue.write().await
            .entry(priority)
            .or_insert_with(Vec::new)
            .push(digest);
        *count += 1;

        Ok(())
    }

    /// Get next batch of transactions
    pub async fn get_batch(&self, max_size: usize) -> Vec<SignedTransaction> {
        let mut batch = Vec::new();
        let mut priority_queue = self.priority_queue.write().await;
        let transactions = self.transactions.read().await;
        let now = Instant::now();

        // Collect transactions by priority
        for (_priority, digests) in priority_queue.iter_mut().rev() {
            for digest in digests.drain(..).collect::<Vec<_>>() {
                if batch.len() >= max_size {
                    break;
                }

                if let Some(info) = transactions.get(&digest) {
                    // Skip expired transactions
                    if now.duration_since(info.insertion_time) > self.config.transaction_timeout {
                        continue;
                    }
                    batch.push(info.transaction.clone());
                }
            }
        }

        batch
    }

    /// Remove transactions
    pub async fn remove_transactions(&self, digests: &[TransactionDigest]) {
        let mut transactions = self.transactions.write().await;
        let mut account_txs = self.account_txs.write().await;

        for digest in digests {
            if let Some(info) = transactions.remove(digest) {
                let sender = info.transaction.sender().to_string();
                if let Some(count) = account_txs.get_mut(&sender) {
                    *count = count.saturating_sub(1);
                }
            }
        }
    }

    /// Garbage collect expired transactions
    pub async fn garbage_collect(&self) {
        let now = Instant::now();
        let mut to_remove = Vec::new();
        let transactions = self.transactions.read().await;

        for (digest, info) in transactions.iter() {
            if now.duration_since(info.insertion_time) > self.config.transaction_timeout {
                to_remove.push(*digest);
            }
        }

        self.remove_transactions(&to_remove).await;
    }
}