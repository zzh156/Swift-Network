// storage/indexes.rs
use super::rocks_store::RocksStore;
use crate::protocol::{ProtocolError, ProtocolResult};
use serde::{Serialize, Deserialize};
use std::sync::Arc;

/// Index key types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IndexKey {
    /// Object index
    Object {
        owner: String,
        type_: String,
    },
    /// Transaction index
    Transaction {
        sender: String,
        timestamp: u64,
    },
    /// Event index
    Event {
        type_: String,
        timestamp: u64,
    },
    /// Custom index
    Custom {
        name: String,
        key: Vec<u8>,
    },
}

/// Index value types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IndexValue {
    /// Object IDs
    ObjectIds(Vec<String>),
    /// Transaction digests
    TransactionDigests(Vec<String>),
    /// Event IDs
    EventIds(Vec<String>),
    /// Custom value
    Custom(Vec<u8>),
}

/// Index store implementation
pub struct IndexStore {
    /// RocksDB store
    rocks: Arc<RocksStore>,
    /// Column family for indexes
    indexes_cf: String,
}

impl IndexStore {
    pub fn new(rocks: Arc<RocksStore>) -> Self {
        Self {
            rocks,
            indexes_cf: "indexes".to_string(),
        }
    }

    /// Get index value
    pub fn get(&self, key: &IndexKey) -> ProtocolResult<Option<IndexValue>> {
        let key_bytes = bincode::serialize(key)?;
        let value_bytes = self.rocks.get(&self.indexes_cf, &key_bytes)?;

        match value_bytes {
            Some(bytes) => {
                let value: IndexValue = bincode::deserialize(&bytes)?;
                Ok(Some(value))
            }
            None => Ok(None),
        }
    }

    /// Update index
    pub fn update(&self, key: IndexKey, value: IndexValue) -> ProtocolResult<()> {
        let key_bytes = bincode::serialize(&key)?;
        let value_bytes = bincode::serialize(&value)?;
        
        self.rocks.put(&self.indexes_cf, &key_bytes, &value_bytes)?;
        Ok(())
    }

    /// Delete index
    pub fn delete(&self, key: &IndexKey) -> ProtocolResult<()> {
        let key_bytes = bincode::serialize(key)?;
        self.rocks.delete(&self.indexes_cf, &key_bytes)?;
        Ok(())
    }

    /// Add to index list
    pub fn add_to_list(&self, key: &IndexKey, id: String) -> ProtocolResult<()> {
        let mut value = match self.get(key)? {
            Some(IndexValue::ObjectIds(mut ids)) => {
                if !ids.contains(&id) {
                    ids.push(id);
                }
                IndexValue::ObjectIds(ids)
            }
            Some(IndexValue::TransactionDigests(mut digests)) => {
                if !digests.contains(&id) {
                    digests.push(id);
                }
                IndexValue::TransactionDigests(digests)
            }
            Some(IndexValue::EventIds(mut ids)) => {
                if !ids.contains(&id) {
                    ids.push(id);
                }
                IndexValue::EventIds(ids)
            }
            Some(IndexValue::Custom(_)) => {
                return Err(ProtocolError::Storage(
                    "Cannot add to custom index".into()
                ))
            }
            None => match key {
                IndexKey::Object { .. } => IndexValue::ObjectIds(vec![id]),
                IndexKey::Transaction { .. } => IndexValue::TransactionDigests(vec![id]),
                IndexKey::Event { .. } => IndexValue::EventIds(vec![id]),
                IndexKey::Custom { .. } => {
                    return Err(ProtocolError::Storage(
                        "Cannot add to custom index".into()
                    ))
                }
            },
        };

        self.update(key.clone(), value)
    }

    /// Remove from index list
    pub fn remove_from_list(&self, key: &IndexKey, id: &str) -> ProtocolResult<()> {
        if let Some(value) = self.get(key)? {
            let new_value = match value {
                IndexValue::ObjectIds(mut ids) => {
                    ids.retain(|x| x != id);
                    IndexValue::ObjectIds(ids)
                }
                IndexValue::TransactionDigests(mut digests) => {
                    digests.retain(|x| x != id);
                    IndexValue::TransactionDigests(digests)
                }
                IndexValue::EventIds(mut ids) => {
                    ids.retain(|x| x != id);
                    IndexValue::EventIds(ids)
                }
                IndexValue::Custom(_) => {
                    return Err(ProtocolError::Storage(
                        "Cannot remove from custom index".into()
                    ))
                }
            };

            self.update(key.clone(), new_value)?;
        }

        Ok(())
    }

    /// Create index iterator
    pub fn iter_prefix(&self, prefix: &[u8]) -> ProtocolResult<impl Iterator<Item = (IndexKey, IndexValue)>> {
        let iter = self.rocks.iter(&self.indexes_cf)?;
        
        Ok(iter
            .filter(move |result| {
                if let Ok((key, _)) = result {
                    key.starts_with(prefix)
                } else {
                    false
                }
            })
            .filter_map(|result| {
                result.ok().and_then(|(key, value)| {
                    let key: IndexKey = bincode::deserialize(&key).ok()?;
                    let value: IndexValue = bincode::deserialize(&value).ok()?;
                    Some((key, value))
                })
            }))
    }

    /// Clear all indexes
    pub fn clear(&self) -> ProtocolResult<()> {
        let batch = self.rocks.batch();
        let iter = self.rocks.iter(&self.indexes_cf)?;

        for item in iter {
            let (key, _) = item?;
            batch.delete(&self.indexes_cf, &key);
        }

        self.rocks.write_batch(batch)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_index_store() -> ProtocolResult<()> {
        let temp_dir = TempDir::new()?;
        let rocks = Arc::new(RocksStore::new(&RocksConfig {
            path: temp_dir.path().to_str().unwrap().to_string(),
            ..Default::default()
        })?);

        let store = IndexStore::new(rocks);

        // Test object index
        let key = IndexKey::Object {
            owner: "alice".to_string(),
            type_: "Coin".to_string(),
        };

        store.add_to_list(&key, "obj1".to_string())?;
        store.add_to_list(&key, "obj2".to_string())?;

        if let Some(IndexValue::ObjectIds(ids)) = store.get(&key)? {
            assert_eq!(ids.len(), 2);
            assert!(ids.contains(&"obj1".to_string()));
            assert!(ids.contains(&"obj2".to_string()));
        }

        // Test remove
        store.remove_from_list(&key, "obj1")?;
        if let Some(IndexValue::ObjectIds(ids)) = store.get(&key)? {
            assert_eq!(ids.len(), 1);
            assert!(ids.contains(&"obj2".to_string()));
        }

        Ok(())
    }
}