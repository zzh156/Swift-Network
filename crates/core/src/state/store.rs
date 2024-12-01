use super::{Checkpoint, StateError, StateResult};
use crate::core::{Object, ObjectID};
use crate::storage::Storage;
use std::sync::Arc;

/// State version
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct StateVersion(pub u64);

/// State store
pub struct StateStore {
    /// Storage
    storage: Arc<dyn Storage>,
}

impl StateStore {
    /// Create new state store
    pub fn new(storage: Arc<dyn Storage>) -> Self {
        Self { storage }
    }

    /// Get object
    pub async fn get_object(
        &self,
        id: &ObjectID,
        version: Option<StateVersion>,
    ) -> StateResult<Option<Object>> {
        self.storage.get_object(id, version)
            .await
            .map_err(|e| StateError::StorageError(e.to_string()))
    }

    /// Put object
    pub async fn put_object(
        &self,
        object: Object,
        version: StateVersion,
    ) -> StateResult<()> {
        self.storage.put_object(&object, version)
            .await
            .map_err(|e| StateError::StorageError(e.to_string()))
    }

    /// Delete object
    pub async fn delete_object(
        &self,
        id: &ObjectID,
        version: StateVersion,
    ) -> StateResult<()> {
        self.storage.delete_object(id, version)
            .await
            .map_err(|e| StateError::StorageError(e.to_string()))
    }

    /// Get checkpoint
    pub async fn get_checkpoint(
        &self,
        sequence: u64,
    ) -> StateResult<Option<Checkpoint>> {
        self.storage.get_checkpoint(sequence)
            .await
            .map_err(|e| StateError::StorageError(e.to_string()))
    }

    /// Put checkpoint
    pub async fn put_checkpoint(
        &self,
        checkpoint: Checkpoint,
    ) -> StateResult<()> {
        self.storage.put_checkpoint(&checkpoint)
            .await
            .map_err(|e| StateError::StorageError(e.to_string()))
    }

    /// Delete checkpoint
    pub async fn delete_checkpoint(
        &self,
        sequence: u64,
    ) -> StateResult<()> {
        self.storage.delete_checkpoint(sequence)
            .await
            .map_err(|e| StateError::StorageError(e.to_string()))
    }

    /// Get latest checkpoint
    pub async fn get_latest_checkpoint(&self) -> StateResult<Option<Checkpoint>> {
        self.storage.get_latest_checkpoint()
            .await
            .map_err(|e| StateError::StorageError(e.to_string()))
    }

    /// Get state at checkpoint
    pub async fn get_state_at_checkpoint(
        &self,
        sequence: u64,
    ) -> StateResult<HashMap<ObjectID, Object>> {
        self.storage.get_state_at_checkpoint(sequence)
            .await
            .map_err(|e| StateError::StorageError(e.to_string()))
    }

    /// Prune state
    pub async fn prune_state(&self, version: StateVersion) -> StateResult<()> {
        self.storage.prune_state(version)
            .await
            .map_err(|e| StateError::StorageError(e.to_string()))
    }
}