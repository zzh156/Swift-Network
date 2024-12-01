use super::{AuthorityError, AuthorityResult, CommitteeInfo};
use crate::storage::{Storage, StorageConfig};
use crate::core::{Object, ObjectID};
use crate::transaction::{Transaction, TransactionDigest, TransactionEffects};
use serde::{Serialize, Deserialize};
use std::sync::Arc;

/// Store configuration 
#[derive(Debug, Clone)]
pub struct StoreConfig {
    /// Storage configuration
    pub storage: StorageConfig,
    /// Cache size
    pub cache_size: usize,
}

/// Authority store
pub struct AuthorityStore {
    /// Storage backend
    storage: Arc<dyn Storage>,
    /// Object cache
    object_cache: Arc<Cache<ObjectID, Object>>,
    /// Transaction cache
    tx_cache: Arc<Cache<TransactionDigest, Transaction>>,
    /// Effects cache
    effects_cache: Arc<Cache<TransactionDigest, TransactionEffects>>,
}

impl AuthorityStore {
    pub fn new(config: StoreConfig) -> AuthorityResult<Self> {
        let storage = Storage::new(config.storage)
            .map_err(|e| AuthorityError::StoreError(e.to_string()))?;

        let object_cache = Cache::new(config.cache_size);
        let tx_cache = Cache::new(config.cache_size);
        let effects_cache = Cache::new(config.cache_size);

        Ok(Self {
            storage: Arc::new(storage),
            object_cache: Arc::new(object_cache),
            tx_cache: Arc::new(tx_cache),
            effects_cache: Arc::new(effects_cache),
        })
    }

    /// Get object
    pub async fn get_object(&self, id: &ObjectID) -> AuthorityResult<Option<Object>> {
        // Try cache first
        if let Some(object) = self.object_cache.get(id) {
            return Ok(Some(object));
        }

        // Get from storage
        let object = self.storage.get_object(&ObjectKey::latest(id))
            .map_err(|e| AuthorityError::StoreError(e.to_string()))?;

        // Update cache
        if let Some(object) = object.clone() {
            self.object_cache.insert(*id, object);
        }

        Ok(object)
    }

    /// Put object
    pub async fn put_object(&self, object: Object) -> AuthorityResult<()> {
        let id = object.id();
        
        // Update storage
        self.storage.put_object(
            ObjectKey::new(id, object.version()),
            object.clone(),
        ).map_err(|e| AuthorityError::StoreError(e.to_string()))?;

        // Update cache
        self.object_cache.insert(id, object);

        Ok(())
    }

    /// Delete object
    pub async fn delete_object(&self, id: &ObjectID) -> AuthorityResult<()> {
        // Delete from storage
        self.storage.delete_object(&ObjectKey::latest(id))
            .map_err(|e| AuthorityError::StoreError(e.to_string()))?;

        // Remove from cache
        self.object_cache.remove(id);

        Ok(())
    }

    /// Get transaction
    pub async fn get_transaction(
        &self,
        digest: &TransactionDigest,
    ) -> AuthorityResult<Option<Transaction>> {
        // Try cache first
        if let Some(tx) = self.tx_cache.get(digest) {
            return Ok(Some(tx));
        }

        // Get from storage
        let tx = self.storage.get_transaction(digest)
            .map_err(|e| AuthorityError::StoreError(e.to_string()))?;

        // Update cache
        if let Some(tx) = tx.clone() {
            self.tx_cache.insert(*digest, tx);
        }

        Ok(tx)
    }

    /// Put transaction
    pub async fn put_transaction(
        &self,
        transaction: Transaction,
    ) -> AuthorityResult<()> {
        let digest = transaction.digest();

        // Update storage
        self.storage.put_transaction(transaction.clone())
            .map_err(|e| AuthorityError::StoreError(e.to_string()))?;

        // Update cache
        self.tx_cache.insert(digest, transaction);

        Ok(())
    }

    /// Get effects
    pub async fn get_effects(
        &self,
        digest: &TransactionDigest,
    ) -> AuthorityResult<Option<TransactionEffects>> {
        // Try cache first
        if let Some(effects) = self.effects_cache.get(digest) {
            return Ok(Some(effects));
        }

        // Get from storage
        let effects = self.storage.get_effects(digest)
            .map_err(|e| AuthorityError::StoreError(e.to_string()))?;

        // Update cache
        if let Some(effects) = effects.clone() {
            self.effects_cache.insert(*digest, effects);
        }

        Ok(effects)
    }

    /// Put effects
    pub async fn put_effects(
        &self,
        effects: TransactionEffects,
    ) -> AuthorityResult<()> {
        let digest = effects.transaction_digest;

        // Update storage
        self.storage.put_effects(effects.clone())
            .map_err(|e| AuthorityError::StoreError(e.to_string()))?;

        // Update cache
        self.effects_cache.insert(digest, effects);

        Ok(())
    }

    /// Get committee info
    pub async fn get_committee(&self) -> AuthorityResult<CommitteeInfo> {
        self.storage.get_committee()
            .map_err(|e| AuthorityError::StoreError(e.to_string()))
    }

    /// Put committee info
    pub async fn put_committee(&self, committee: CommitteeInfo) -> AuthorityResult<()> {
        self.storage.put_committee(committee)
            .map_err(|e| AuthorityError::StoreError(e.to_string()))
    }

    /// Clear caches
    pub fn clear_caches(&self) {
        self.object_cache.clear();
        self.tx_cache.clear();
        self.effects_cache.clear();
    }
}

/// Object key for storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectKey {
    /// Object ID
    pub id: ObjectID,
    /// Version
    pub version: SequenceNumber,
}

impl ObjectKey {
    pub fn new(id: ObjectID, version: SequenceNumber) -> Self {
        Self { id, version }
    }

    pub fn latest(id: &ObjectID) -> Self {
        Self {
            id: *id,
            version: SequenceNumber::MAX,
        }
    }
}