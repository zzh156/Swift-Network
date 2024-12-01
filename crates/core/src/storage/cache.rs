// storage/cache.rs
use crate::protocol::{ProtocolError, ProtocolResult};
use moka::sync::Cache;
use std::time::Duration;
use serde::{Serialize, Deserialize};

/// Cache configuration
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// Maximum cache size
    pub max_capacity: u64,
    /// Time to live
    pub ttl: Duration,
    /// Time to idle
    pub tti: Duration,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_capacity: 10_000,
            ttl: Duration::from_secs(3600),    // 1 hour
            tti: Duration::from_secs(300),     // 5 minutes
        }
    }
}

/// Cache value wrapper
#[derive(Clone, Serialize, Deserialize)]
pub struct CacheValue {
    /// Raw data
    pub data: Vec<u8>,
    /// Value type
    pub type_: String,
    /// Creation time
    pub created_at: u64,
}

/// Cache store implementation
pub struct CacheStore {
    /// Inner cache
    cache: Cache<String, CacheValue>,
    /// Configuration
    config: CacheConfig,
}

impl CacheStore {
    pub fn new(config: CacheConfig) -> Self {
        let cache = Cache::builder()
            .max_capacity(config.max_capacity)
            .time_to_live(config.ttl)
            .time_to_idle(config.tti)
            .build();

        Self { cache, config }
    }

    /// Get value from cache
    pub fn get(&self, key: &str) -> ProtocolResult<Option<CacheValue>> {
        Ok(self.cache.get(key))
    }

    /// Put value into cache
    pub fn put(&self, key: String, value: CacheValue) -> ProtocolResult<()> {
        self.cache.insert(key, value);
        Ok(())
    }

    /// Delete value from cache
    pub fn delete(&self, key: &str) -> ProtocolResult<()> {
        self.cache.invalidate(key);
        Ok(())
    }

    /// Get multiple values
    pub fn get_many(&self, keys: &[String]) -> ProtocolResult<Vec<Option<CacheValue>>> {
        Ok(keys.iter().map(|key| self.cache.get(key)).collect())
    }

    /// Put multiple values
    pub fn put_many(&self, entries: Vec<(String, CacheValue)>) -> ProtocolResult<()> {
        for (key, value) in entries {
            self.cache.insert(key, value);
        }
        Ok(())
    }

    /// Delete multiple values
    pub fn delete_many(&self, keys: &[String]) -> ProtocolResult<()> {
        for key in keys {
            self.cache.invalidate(key);
        }
        Ok(())
    }

    /// Clear all entries
    pub fn clear(&self) -> ProtocolResult<()> {
        self.cache.invalidate_all();
        Ok(())
    }

    /// Get cache size
    pub fn size(&self) -> u64 {
        self.cache.entry_count()
    }

    /// Check if key exists
    pub fn contains(&self, key: &str) -> bool {
        self.cache.contains_key(key)
    }

    /// Get cache stats
    pub fn stats(&self) -> CacheStats {
        CacheStats {
            size: self.size(),
            max_capacity: self.config.max_capacity,
            hit_count: self.cache.hit_count(),
            miss_count: self.cache.miss_count(),
            eviction_count: self.cache.eviction_count(),
        }
    }

    /// Add cache listener
    pub fn add_listener<F>(&self, listener: F)
    where
        F: Fn(CacheEvent) + Send + Sync + 'static,
    {
        self.cache
            .clone()
            .with_invalidation_listener(move |key, _value, cause| {
                listener(CacheEvent::Invalidated {
                    key: key.clone(),
                    cause: cause.into(),
                });
            });
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    /// Current size
    pub size: u64,
    /// Maximum capacity
    pub max_capacity: u64,
    /// Hit count
    pub hit_count: u64,
    /// Miss count
    pub miss_count: u64,
    /// Eviction count
    pub eviction_count: u64,
}

/// Cache events
#[derive(Debug, Clone)]
pub enum CacheEvent {
    /// Entry invalidated
    Invalidated {
        /// Key
        key: String,
        /// Cause
        cause: InvalidationCause,
    },
}

/// Invalidation causes
#[derive(Debug, Clone)]
pub enum InvalidationCause {
    /// Explicit removal
    Explicit,
    /// Size eviction
    Size,
    /// Time to live expired
    Expired,
    /// Time to idle expired
    Idle,
}

impl From<moka::notification::RemovalCause> for InvalidationCause {
    fn from(cause: moka::notification::RemovalCause) -> Self {
        match cause {
            moka::notification::RemovalCause::Explicit => InvalidationCause::Explicit,
            moka::notification::RemovalCause::Size => InvalidationCause::Size,
            moka::notification::RemovalCause::Expired => InvalidationCause::Expired,
            moka::notification::RemovalCause::Idle => InvalidationCause::Idle,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_cache_store() -> ProtocolResult<()> {
        let config = CacheConfig {
            max_capacity: 100,
            ttl: Duration::from_secs(1),
            tti: Duration::from_secs(1),
        };

        let store = CacheStore::new(config);

        // Test put and get
        let value = CacheValue {
            data: vec![1, 2, 3],
            type_: "test".to_string(),
            created_at: 100,
        };

        store.put("key1".to_string(), value.clone())?;
        let retrieved = store.get("key1")?;
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().data, value.data);

        // Test expiration
        thread::sleep(Duration::from_secs(2));
        let expired = store.get("key1")?;
        assert!(expired.is_none());

        // Test multiple operations
        let entries = vec![
            ("key2".to_string(), value.clone()),
            ("key3".to_string(), value.clone()),
        ];
        store.put_many(entries)?;

        let keys = vec!["key2".to_string(), "key3".to_string()];
        let values = store.get_many(&keys)?;
        assert_eq!(values.len(), 2);
        assert!(values.iter().all(|v| v.is_some()));

        // Test stats
        let stats = store.stats();
        assert_eq!(stats.size, 2);
        assert!(stats.hit_count > 0);

        Ok(())
    }
}