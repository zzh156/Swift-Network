//! Storage module for managing blockchain state and data persistence.

mod object_store;
mod event_store;
mod rocks_store;
mod indexes;
mod cache;

pub use object_store::{ObjectStore, ObjectKey, ObjectValue};
pub use event_store::{EventStore, Event, EventFilter};
pub use rocks_store::{RocksStore, RocksConfig};
pub use indexes::{IndexStore, IndexKey, IndexValue};
pub use cache::{CacheStore, CacheConfig};

use crate::protocol::{ProtocolError, ProtocolResult};
use std::sync::Arc;

/// Storage configuration
#[derive(Debug, Clone)]
pub struct StorageConfig {
    /// Data directory
    pub data_dir: String,
    /// Cache configuration
    pub cache_config: CacheConfig,
    /// RocksDB configuration
    pub rocks_config: RocksConfig,
}

/// Main storage interface
pub trait Storage: Send + Sync {
    /// Get object by key
    fn get_object(&self, key: &ObjectKey) -> ProtocolResult<Option<ObjectValue>>;
    
    /// Put object
    fn put_object(&self, key: ObjectKey, value: ObjectValue) -> ProtocolResult<()>;
    
    /// Delete object
    fn delete_object(&self, key: &ObjectKey) -> ProtocolResult<()>;
    
    /// Get events by filter
    fn get_events(&self, filter: &EventFilter) -> ProtocolResult<Vec<Event>>;
    
    /// Emit event
    fn emit_event(&self, event: Event) -> ProtocolResult<()>;
    
    /// Get index value
    fn get_index(&self, key: &IndexKey) -> ProtocolResult<Option<IndexValue>>;
    
    /// Update index
    fn update_index(&self, key: IndexKey, value: IndexValue) -> ProtocolResult<()>;
}

/// Storage manager
pub struct StorageManager {
    /// Object store
    object_store: Arc<ObjectStore>,
    /// Event store
    event_store: Arc<EventStore>,
    /// Index store
    index_store: Arc<IndexStore>,
    /// Cache store
    cache_store: Arc<CacheStore>,
}

impl StorageManager {
    pub fn new(config: StorageConfig) -> ProtocolResult<Self> {
        // Initialize RocksDB
        let rocks = RocksStore::new(&config.rocks_config)?;
        
        // Create stores
        let object_store = Arc::new(ObjectStore::new(rocks.clone()));
        let event_store = Arc::new(EventStore::new(rocks.clone()));
        let index_store = Arc::new(IndexStore::new(rocks.clone()));
        let cache_store = Arc::new(CacheStore::new(config.cache_config));
        
        Ok(Self {
            object_store,
            event_store,
            index_store,
            cache_store,
        })
    }
    
    pub fn object_store(&self) -> Arc<ObjectStore> {
        self.object_store.clone()
    }
    
    pub fn event_store(&self) -> Arc<EventStore> {
        self.event_store.clone()
    }
    
    pub fn index_store(&self) -> Arc<IndexStore> {
        self.index_store.clone()
    }
    
    pub fn cache_store(&self) -> Arc<CacheStore> {
        self.cache_store.clone()
    }
}

impl Storage for StorageManager {
    fn get_object(&self, key: &ObjectKey) -> ProtocolResult<Option<ObjectValue>> {
        // Try cache first
        if let Some(value) = self.cache_store.get(key)? {
            return Ok(Some(value));
        }
        
        // Get from object store
        let value = self.object_store.get(key)?;
        
        // Update cache
        if let Some(value) = value.clone() {
            self.cache_store.put(key.clone(), value)?;
        }
        
        Ok(value)
    }
    
    fn put_object(&self, key: ObjectKey, value: ObjectValue) -> ProtocolResult<()> {
        // Update object store
        self.object_store.put(key.clone(), value.clone())?;
        
        // Update cache
        self.cache_store.put(key, value)?;
        
        Ok(())
    }
    
    fn delete_object(&self, key: &ObjectKey) -> ProtocolResult<()> {
        // Delete from object store
        self.object_store.delete(key)?;
        
        // Delete from cache
        self.cache_store.delete(key)?;
        
        Ok(())
    }
    
    fn get_events(&self, filter: &EventFilter) -> ProtocolResult<Vec<Event>> {
        self.event_store.get_events(filter)
    }
    
    fn emit_event(&self, event: Event) -> ProtocolResult<()> {
        self.event_store.emit_event(event)
    }
    
    fn get_index(&self, key: &IndexKey) -> ProtocolResult<Option<IndexValue>> {
        self.index_store.get(key)
    }
    
    fn update_index(&self, key: IndexKey, value: IndexValue) -> ProtocolResult<()> {
        self.index_store.update(key, value)
    }
}