use crate::storage::{Storage, StorageConfig};
use crate::protocol::{ProtocolError, ProtocolResult};
use serde::{Serialize, Deserialize};
use std::sync::Arc;

/// Index key types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IndexKey {
    /// Transaction index
    Transaction {
        hash: TransactionDigest,
    },
    /// Transaction effects index
    TransactionEffects {
        transaction_hash: TransactionDigest,
    },
    /// Object index
    Object {
        id: ObjectID,
    },
    /// Event index
    Event {
        type_: String,
        timestamp: u64,
    },
    /// Address index
    Address {
        address: Address,
        type_: AddressIndexType,
        timestamp: u64,
    },
}

/// Address index types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AddressIndexType {
    /// Transaction
    Transaction,
    /// Object
    Object,
}

/// Index value types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IndexValue {
    /// Transaction
    Transaction(SignedTransaction),
    /// Transaction digest
    TransactionDigest(TransactionDigest),
    /// Transaction effects
    TransactionEffects(TransactionEffects),
    /// Object
    Object(Object),
    /// Object ID
    ObjectId(ObjectID),
    /// Event
    Event(Event),
}

/// Index store implementation
pub struct IndexStore {
    /// Storage
    storage: Arc<dyn Storage>,
    /// Column family for indexes
    indexes_cf: String,
}

impl IndexStore {
    pub fn new(config: StorageConfig) -> ProtocolResult<Self> {
        let storage = Storage::new(config)?;
        
        Ok(Self {
            storage: Arc::new(storage),
            indexes_cf: "indexes".to_string(),
        })
    }

    /// Get value by key
    pub async fn get(&self, key: &IndexKey) -> ProtocolResult<Option<IndexValue>> {
        let key_bytes = bincode::serialize(key)?;
        let value_bytes = self.storage.get(&self.indexes_cf, &key_bytes)?;

        match value_bytes {
            Some(bytes) => {
                let value: IndexValue = bincode::deserialize(&bytes)?;
                Ok(Some(value))
            }
            None => Ok(None),
        }
    }

    /// Put key-value pair
    pub async fn put(&self, key: IndexKey, value: IndexValue) -> ProtocolResult<()> {
        let key_bytes = bincode::serialize(&key)?;
        let value_bytes = bincode::serialize(&value)?;

        self.storage.put(&self.indexes_cf, key_bytes, value_bytes)?;
        Ok(())
    }

    /// Put batch of key-value pairs
    pub async fn put_batch(
        &self,
        entries: Vec<(IndexKey, IndexValue)>,
    ) -> ProtocolResult<()> {
        let mut batch = self.storage.batch();

        for (key, value) in entries {
            let key_bytes = bincode::serialize(&key)?;
            let value_bytes = bincode::serialize(&value)?;
            batch.put(&self.indexes_cf, &key_bytes, &value_bytes);
        }

        self.storage.write_batch(batch)?;
        Ok(())
    }

    /// Delete by key
    pub async fn delete(&self, key: &IndexKey) -> ProtocolResult<()> {
        let key_bytes = bincode::serialize(key)?;
        self.storage.delete(&self.indexes_cf, &key_bytes)?;
        Ok(())
    }

    /// Create prefix iterator
    pub async fn iter_prefix(&self, prefix: &[u8]) -> ProtocolResult<IndexIterator> {
        let iter = self.storage.iter_prefix(&self.indexes_cf, prefix)?;
        Ok(IndexIterator { inner: iter })
    }

    /// Clear all indexes
    pub async fn clear(&self) -> ProtocolResult<()> {
        let batch = self.storage.batch();
        let iter = self.storage.iter(&self.indexes_cf)?;

        for item in iter {
            let (key, _) = item?;
            batch.delete(&self.indexes_cf, &key);
        }

        self.storage.write_batch(batch)?;
        Ok(())
    }
}

/// Index iterator
pub struct IndexIterator {
    inner: Box<dyn Iterator<Item = ProtocolResult<(Vec<u8>, Vec<u8>)>>>,
}

impl IndexIterator {
    /// Get next item
    pub async fn next(&mut self) -> ProtocolResult<Option<(IndexKey, IndexValue)>> {
        match self.inner.next() {
            Some(result) => {
                let (key_bytes, value_bytes) = result?;
                let key: IndexKey = bincode::deserialize(&key_bytes)?;
                let value: IndexValue = bincode::deserialize(&value_bytes)?;
                Ok(Some((key, value)))
            }
            None => Ok(None),
        }
    }

    /// Seek to key
    pub fn seek(&mut self, key: &str) -> ProtocolResult<()> {
        // Implementation depends on storage backend
        Ok(())
    }

    /// Reverse iterator direction
    pub fn reverse(&mut self) {
        // Implementation depends on storage backend
    }
}

impl IndexKey {
    /// Create address prefix
    pub fn address_prefix(address: &Address, type_: AddressIndexType) -> Vec<u8> {
        let prefix = IndexKey::Address {
            address: *address,
            type_,
            timestamp: 0,
        };
        bincode::serialize(&prefix).unwrap()
    }

    /// Create event prefix
    pub fn event_prefix(type_: &str) -> Vec<u8> {
        let prefix = IndexKey::Event {
            type_: type_.to_string(),
            timestamp: 0,
        };
        bincode::serialize(&prefix).unwrap()
    }
}