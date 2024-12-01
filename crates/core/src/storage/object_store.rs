// storage/object_store.rs
use super::rocks_store::RocksStore;
use crate::protocol::{ProtocolError, ProtocolResult};
use crate::core::{ObjectID, SequenceNumber};
use serde::{Serialize, Deserialize};
use std::sync::Arc;

/// Object key for storage
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct ObjectKey {
    /// Object ID
    pub id: ObjectID,
    /// Version number
    pub version: SequenceNumber,
}

/// Object value in storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectValue {
    /// Object data
    pub data: Vec<u8>,
    /// Owner address
    pub owner: String,
    /// Object type
    pub type_: String,
    /// Creation timestamp
    pub created_at: u64,
    /// Last modified timestamp
    pub modified_at: u64,
}

/// Object metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectMetadata {
    /// Latest version
    pub latest_version: SequenceNumber,
    /// Deletion marker
    pub deleted: bool,
    /// Reference count
    pub ref_count: u64,
}

/// Object store implementation
pub struct ObjectStore {
    /// RocksDB store
    rocks: Arc<RocksStore>,
    /// Column family for objects
    objects_cf: String,
    /// Column family for metadata
    metadata_cf: String,
}

impl ObjectStore {
    pub fn new(rocks: Arc<RocksStore>) -> Self {
        Self {
            rocks,
            objects_cf: "objects".to_string(),
            metadata_cf: "object_metadata".to_string(),
        }
    }

    /// Get object by key
    pub fn get(&self, key: &ObjectKey) -> ProtocolResult<Option<ObjectValue>> {
        // Check metadata first
        let metadata = self.get_metadata(&key.id)?;
        if let Some(meta) = metadata {
            if meta.deleted {
                return Ok(None);
            }
        }

        // Get from rocks
        let key_bytes = bincode::serialize(key)?;
        let value_bytes = self.rocks.get(&self.objects_cf, &key_bytes)?;

        match value_bytes {
            Some(bytes) => {
                let value: ObjectValue = bincode::deserialize(&bytes)?;
                Ok(Some(value))
            }
            None => Ok(None),
        }
    }

    /// Put object
    pub fn put(&self, key: ObjectKey, value: ObjectValue) -> ProtocolResult<()> {
        // Update metadata
        let mut metadata = self.get_metadata(&key.id)?.unwrap_or_else(|| ObjectMetadata {
            latest_version: key.version,
            deleted: false,
            ref_count: 0,
        });
        
        if key.version > metadata.latest_version {
            metadata.latest_version = key.version;
        }
        metadata.ref_count += 1;

        // Write object
        let key_bytes = bincode::serialize(&key)?;
        let value_bytes = bincode::serialize(&value)?;
        
        let batch = self.rocks.batch();
        batch.put(&self.objects_cf, &key_bytes, &value_bytes)?;
        
        // Write metadata
        let metadata_key = bincode::serialize(&key.id)?;
        let metadata_value = bincode::serialize(&metadata)?;
        batch.put(&self.metadata_cf, &metadata_key, &metadata_value)?;
        
        batch.write()?;
        
        Ok(())
    }

    /// Delete object
    pub fn delete(&self, key: &ObjectKey) -> ProtocolResult<()> {
        // Update metadata
        if let Some(mut metadata) = self.get_metadata(&key.id)? {
            metadata.deleted = true;
            metadata.ref_count = metadata.ref_count.saturating_sub(1);

            let metadata_key = bincode::serialize(&key.id)?;
            let metadata_value = bincode::serialize(&metadata)?;

            let batch = self.rocks.batch();
            batch.put(&self.metadata_cf, &metadata_key, &metadata_value)?;
            
            // Delete object
            let key_bytes = bincode::serialize(key)?;
            batch.delete(&self.objects_cf, &key_bytes)?;
            
            batch.write()?;
        }

        Ok(())
    }

    /// Get object metadata
    fn get_metadata(&self, id: &ObjectID) -> ProtocolResult<Option<ObjectMetadata>> {
        let key = bincode::serialize(id)?;
        let value = self.rocks.get(&self.metadata_cf, &key)?;

        match value {
            Some(bytes) => {
                let metadata: ObjectMetadata = bincode::deserialize(&bytes)?;
                Ok(Some(metadata))
            }
            None => Ok(None),
        }
    }

    /// List all objects
    pub fn list(&self) -> ProtocolResult<Vec<(ObjectKey, ObjectValue)>> {
        let mut objects = Vec::new();
        let iter = self.rocks.iter(&self.objects_cf)?;

        for item in iter {
            let (key_bytes, value_bytes) = item?;
            let key: ObjectKey = bincode::deserialize(&key_bytes)?;
            let value: ObjectValue = bincode::deserialize(&value_bytes)?;
            objects.push((key, value));
        }

        Ok(objects)
    }

    /// Get latest version of object
    pub fn get_latest_version(&self, id: &ObjectID) -> ProtocolResult<Option<SequenceNumber>> {
        Ok(self.get_metadata(id)?.map(|m| m.latest_version))
    }

    /// Check if object exists
    pub fn exists(&self, id: &ObjectID) -> ProtocolResult<bool> {
        Ok(self.get_metadata(id)?.is_some())
    }

    /// Get reference count
    pub fn get_ref_count(&self, id: &ObjectID) -> ProtocolResult<u64> {
        Ok(self.get_metadata(id)?.map(|m| m.ref_count).unwrap_or(0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_object_store() -> ProtocolResult<()> {
        let temp_dir = TempDir::new()?;
        let rocks = Arc::new(RocksStore::new(&RocksConfig {
            path: temp_dir.path().to_str().unwrap().to_string(),
            ..Default::default()
        })?);

        let store = ObjectStore::new(rocks);

        // Test put and get
        let key = ObjectKey {
            id: ObjectID::random(),
            version: SequenceNumber::new(1),
        };

        let value = ObjectValue {
            data: vec![1, 2, 3],
            owner: "test".to_string(),
            type_: "TestObject".to_string(),
            created_at: 100,
            modified_at: 100,
        };

        store.put(key.clone(), value.clone())?;
        let retrieved = store.get(&key)?.unwrap();
        assert_eq!(retrieved.data, value.data);

        // Test delete
        store.delete(&key)?;
        assert!(store.get(&key)?.is_none());

        Ok(())
    }
}