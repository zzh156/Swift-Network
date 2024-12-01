use super::{AuthorityError, AuthorityResult, AuthorityStore};
use crate::core::{Object, ObjectID};
use crate::transaction::{Transaction, TransactionDigest, TransactionEffects};
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
    /// Effects included
    pub effects: Vec<TransactionEffects>,
    /// State root
    pub state_root: [u8; 32],
    /// Epoch
    pub epoch: u64,
    /// Next epoch committee
    pub next_epoch_committee: Option<CommitteeInfo>,
}

impl Checkpoint {
    /// Create new checkpoint
    pub fn new(
        sequence: u64,
        previous_digest: Option<[u8; 32]>,
        timestamp: u64,
        transactions: Vec<TransactionDigest>,
        effects: Vec<TransactionEffects>,
        state_root: [u8; 32],
        epoch: u64,
        next_epoch_committee: Option<CommitteeInfo>,
    ) -> Self {
        let mut checkpoint = Self {
            sequence,
            digest: [0; 32],
            previous_digest,
            timestamp,
            transactions,
            effects,
            state_root,
            epoch,
            next_epoch_committee,
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
            hasher.update(tx.as_ref());
        }
        
        for effect in &self.effects {
            hasher.update(effect.transaction_digest.as_ref());
        }
        
        hasher.update(self.state_root);
        hasher.update(self.epoch.to_le_bytes());
        
        hasher.finalize().into()
    }

    /// Verify checkpoint
    pub fn verify(&self) -> bool {
        self.digest == self.compute_digest()
    }
}

/// Checkpoint store
pub struct CheckpointStore {
    /// Authority store
    store: Arc<AuthorityStore>,
    /// Column family for checkpoints
    checkpoints_cf: String,
}

impl CheckpointStore {
    pub fn new(store: Arc<AuthorityStore>) -> AuthorityResult<Self> {
        Ok(Self {
            store,
            checkpoints_cf: "checkpoints".to_string(),
        })
    }

    /// Get checkpoint by sequence
    pub async fn get_checkpoint(
        &self,
        sequence: u64,
    ) -> AuthorityResult<Option<Checkpoint>> {
        let key = sequence.to_le_bytes();
        let value = self.store.storage()
            .get(&self.checkpoints_cf, &key)
            .map_err(|e| AuthorityError::StoreError(e.to_string()))?;

        match value {
            Some(bytes) => {
                let checkpoint: Checkpoint = bincode::deserialize(&bytes)
                    .map_err(|e| AuthorityError::DeserializationError(e.to_string()))?;
                Ok(Some(checkpoint))
            }
            None => Ok(None),
        }
    }

    /// Put checkpoint
    pub async fn put_checkpoint(&self, checkpoint: Checkpoint) -> AuthorityResult<()> {
        // Verify checkpoint first
        if !checkpoint.verify() {
            return Err(AuthorityError::InvalidCheckpoint(
                "Invalid checkpoint digest".into()
            ));
        }

        let key = checkpoint.sequence.to_le_bytes();
        let value = bincode::serialize(&checkpoint)
            .map_err(|e| AuthorityError::SerializationError(e.to_string()))?;

        self.store.storage()
            .put(&self.checkpoints_cf, &key, &value)
            .map_err(|e| AuthorityError::StoreError(e.to_string()))?;

        Ok(())
    }

    /// Get latest checkpoint
    pub async fn get_latest_checkpoint(&self) -> AuthorityResult<Option<Checkpoint>> {
        let mut iter = self.store.storage()
            .iter(&self.checkpoints_cf)
            .map_err(|e| AuthorityError::StoreError(e.to_string()))?;

        iter.seek_to_last();

        if let Some(Ok((_, value))) = iter.next() {
            let checkpoint: Checkpoint = bincode::deserialize(&value)
                .map_err(|e| AuthorityError::DeserializationError(e.to_string()))?;
            Ok(Some(checkpoint))
        } else {
            Ok(None)
        }
    }

    /// Get checkpoint range
    pub async fn get_checkpoint_range(
        &self,
        start: u64,
        end: u64,
    ) -> AuthorityResult<Vec<Checkpoint>> {
        let mut checkpoints = Vec::new();
        let mut iter = self.store.storage()
            .iter(&self.checkpoints_cf)
            .map_err(|e| AuthorityError::StoreError(e.to_string()))?;

        iter.seek(&start.to_le_bytes());

        while let Some(Ok((key, value))) = iter.next() {
            let sequence = u64::from_le_bytes(key.try_into().unwrap());
            if sequence > end {
                break;
            }

            let checkpoint: Checkpoint = bincode::deserialize(&value)
                .map_err(|e| AuthorityError::DeserializationError(e.to_string()))?;
            checkpoints.push(checkpoint);
        }

        Ok(checkpoints)
    }

    /// Get state at checkpoint
    pub async fn get_state_at_checkpoint(
        &self,
        sequence: u64,
    ) -> AuthorityResult<HashMap<ObjectID, Object>> {
        let checkpoint = self.get_checkpoint(sequence).await?
            .ok_or_else(|| AuthorityError::CheckpointNotFound(sequence))?;

        let mut state = HashMap::new();
        for effect in checkpoint.effects {
            // Add created objects
            for (id, object) in effect.created_objects {
                state.insert(id, object);
            }

            // Update modified objects
            for (id, object) in effect.modified_objects {
                state.insert(id, object);
            }

            // Remove deleted objects
            for id in effect.deleted_objects {
                state.remove(&id);
            }
        }

        Ok(state)
    }
}