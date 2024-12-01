use super::{StateError, StateResult, StateStore};
use crate::core::{Object, ObjectID};
use crate::protocol::TransactionDigest;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

/// Checkpoint data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Checkpoint {
    /// Sequence number
    pub sequence: u64,
    /// Content digest
    pub digest: [u8; 32],
    /// Previous checkpoint digest
    pub previous_digest: Option<[u8; 32]>,
    /// Timestamp
    pub timestamp: u64,
    /// Transactions included
    pub transactions: Vec<TransactionDigest>,
    /// State root
    pub state_root: [u8; 32],
    /// Epoch
    pub epoch: u64,
}

impl Checkpoint {
    /// Create new checkpoint
    pub fn new(
        sequence: u64,
        previous_digest: Option<[u8; 32]>,
        timestamp: u64,
        transactions: Vec<TransactionDigest>,
        state_root: [u8; 32],
        epoch: u64,
    ) -> Self {
        let mut checkpoint = Self {
            sequence,
            digest: [0; 32],
            previous_digest,
            timestamp,
            transactions,
            state_root,
            epoch,
        };
        checkpoint.digest = checkpoint.compute_digest();
        checkpoint
    }

    /// Compute checkpoint digest
    fn compute_digest(&self) -> [u8; 32] {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(self.sequence.to_le_bytes());
        if let Some(prev) = self.previous_digest {
            hasher.update(prev);
        }
        hasher.update(self.timestamp.to_le_bytes());
        for tx in &self.transactions {
            hasher.update(tx.as_bytes());
        }
        hasher.update(self.state_root);
        hasher.update(self.epoch.to_le_bytes());
        hasher.finalize().into()
    }
}

/// Checkpoint store
pub struct CheckpointStore {
    /// State store
    store: Arc<StateStore>,
}

impl CheckpointStore {
    /// Create new checkpoint store
    pub fn new(store: Arc<StateStore>) -> Self {
        Self { store }
    }

    /// Get checkpoint by sequence
    pub async fn get_checkpoint(
        &self,
        sequence: u64,
    ) -> StateResult<Option<Checkpoint>> {
        self.store.get_checkpoint(sequence).await
    }

    /// Put checkpoint
    pub async fn put_checkpoint(
        &self,
        checkpoint: Checkpoint,
    ) -> StateResult<()> {
        // Verify checkpoint
        if checkpoint.digest != checkpoint.compute_digest() {
            return Err(StateError::InvalidState("Invalid checkpoint digest".into()));
        }

        // Store checkpoint
        self.store.put_checkpoint(checkpoint).await
    }

    /// Get latest checkpoint
    pub async fn get_latest_checkpoint(&self) -> StateResult<Option<Checkpoint>> {
        self.store.get_latest_checkpoint().await
    }

    /// Get state at checkpoint
    pub async fn get_state_at_checkpoint(
        &self,
        sequence: u64,
    ) -> StateResult<HashMap<ObjectID, Object>> {
        self.store.get_state_at_checkpoint(sequence).await
    }
}