// storage/rocks_store.rs
use crate::protocol::{ProtocolError, ProtocolResult};
use rocksdb::{
    ColumnFamily, ColumnFamilyDescriptor, DBCompactionStyle, DBCompressionType,
    Options, WriteBatch, DB,
};
use std::path::Path;
use std::sync::Arc;

/// RocksDB configuration
#[derive(Debug, Clone)]
pub struct RocksConfig {
    /// Data directory path
    pub path: String,
    /// Write buffer size
    pub write_buffer_size: usize,
    /// Max write buffer number
    pub max_write_buffer_number: i32,
    /// Max background jobs
    pub max_background_jobs: i32,
    /// Block cache size
    pub block_cache_size: usize,
    /// Compression type
    pub compression_type: DBCompressionType,
    /// Compaction style
    pub compaction_style: DBCompactionStyle,
}

impl Default for RocksConfig {
    fn default() -> Self {
        Self {
            path: "data".to_string(),
            write_buffer_size: 64 * 1024 * 1024,    // 64MB
            max_write_buffer_number: 4,
            max_background_jobs: 4,
            block_cache_size: 512 * 1024 * 1024,    // 512MB
            compression_type: DBCompressionType::Lz4,
            compaction_style: DBCompactionStyle::Level,
        }
    }
}

/// RocksDB store implementation
pub struct RocksStore {
    /// RocksDB instance
    db: Arc<DB>,
    /// Column families
    column_families: Vec<String>,
}

impl RocksStore {
    /// Create new RocksDB store
    pub fn new(config: &RocksConfig) -> ProtocolResult<Self> {
        // Create options
        let mut opts = Options::default();
        opts.create_if_missing(true);
        opts.create_missing_column_families(true);
        opts.set_write_buffer_size(config.write_buffer_size);
        opts.set_max_write_buffer_number(config.max_write_buffer_number);
        opts.set_max_background_jobs(config.max_background_jobs);
        opts.set_compression_type(config.compression_type);
        opts.set_compaction_style(config.compaction_style);

        // Set block cache
        let cache = rocksdb::Cache::new_lru_cache(config.block_cache_size)?;
        let mut block_opts = rocksdb::BlockBasedOptions::default();
        block_opts.set_block_cache(&cache);
        opts.set_block_based_table_factory(&block_opts);

        // Default column families
        let cf_names = vec![
            "default",
            "objects",
            "object_metadata",
            "events",
            "event_indexes",
            "transactions",
            "state",
        ];

        // Create column family descriptors
        let cf_descriptors: Vec<_> = cf_names
            .iter()
            .map(|name| ColumnFamilyDescriptor::new(*name, opts.clone()))
            .collect();

        // Open database
        let db = DB::open_cf_descriptors(&opts, &config.path, cf_descriptors)?;
        let db = Arc::new(db);

        Ok(Self {
            db,
            column_families: cf_names.into_iter().map(String::from).collect(),
        })
    }

    /// Get value by key
    pub fn get(&self, cf: &str, key: &[u8]) -> ProtocolResult<Option<Vec<u8>>> {
        let cf = self.get_cf(cf)?;
        Ok(self.db.get_cf(cf, key)?)
    }

    /// Put key-value pair
    pub fn put(&self, cf: &str, key: &[u8], value: &[u8]) -> ProtocolResult<()> {
        let cf = self.get_cf(cf)?;
        Ok(self.db.put_cf(cf, key, value)?)
    }

    /// Delete key
    pub fn delete(&self, cf: &str, key: &[u8]) -> ProtocolResult<()> {
        let cf = self.get_cf(cf)?;
        Ok(self.db.delete_cf(cf, key)?)
    }

    /// Create write batch
    pub fn batch(&self) -> WriteBatch {
        WriteBatch::default()
    }

    /// Write batch
    pub fn write_batch(&self, batch: WriteBatch) -> ProtocolResult<()> {
        Ok(self.db.write(batch)?)
    }

    /// Create iterator
    pub fn iter(&self, cf: &str) -> ProtocolResult<rocksdb::DBIterator> {
        let cf = self.get_cf(cf)?;
        Ok(self.db.iterator_cf(cf, rocksdb::IteratorMode::Start))
    }

    /// Get column family handle
    fn get_cf(&self, name: &str) -> ProtocolResult<&ColumnFamily> {
        self.db
            .cf_handle(name)
            .ok_or_else(|| ProtocolError::Storage(format!("Column family not found: {}", name)))
    }

    /// Create new column family
    pub fn create_cf(&mut self, name: &str) -> ProtocolResult<()> {
        if !self.column_families.contains(&name.to_string()) {
            self.db.create_cf(name, &Options::default())?;
            self.column_families.push(name.to_string());
        }
        Ok(())
    }

    /// Drop column family
    pub fn drop_cf(&mut self, name: &str) -> ProtocolResult<()> {
        if self.column_families.contains(&name.to_string()) {
            self.db.drop_cf(name)?;
            self.column_families.retain(|cf| cf != name);
        }
        Ok(())
    }

    /// Get snapshot
    pub fn snapshot(&self) -> rocksdb::Snapshot {
        self.db.snapshot()
    }

    /// Compact range
    pub fn compact_range(
        &self,
        cf: &str,
        start: Option<&[u8]>,
        end: Option<&[u8]>,
    ) -> ProtocolResult<()> {
        let cf = self.get_cf(cf)?;
        self.db.compact_range_cf(cf, start, end);
        Ok(())
    }

    /// Flush
    pub fn flush(&self) -> ProtocolResult<()> {
        Ok(self.db.flush()?)
    }

    /// Get statistics
    pub fn get_statistics(&self) -> Option<String> {
        self.db.property_value("rocksdb.stats")
    }

    /// Get approximate size
    pub fn get_approximate_size(
        &self,
        cf: &str,
        start: &[u8],
        end: &[u8],
    ) -> ProtocolResult<u64> {
        let cf = self.get_cf(cf)?;
        let sizes = self.db.get_approximate_sizes_cf(cf, &[(start, end)]);
        Ok(sizes[0])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_rocks_store() -> ProtocolResult<()> {
        let temp_dir = TempDir::new()?;
        let config = RocksConfig {
            path: temp_dir.path().to_str().unwrap().to_string(),
            ..Default::default()
        };

        let store = RocksStore::new(&config)?;

        // Test put and get
        store.put("default", b"key1", b"value1")?;
        let value = store.get("default", b"key1")?;
        assert_eq!(value, Some(b"value1".to_vec()));

        // Test delete
        store.delete("default", b"key1")?;
        let value = store.get("default", b"key1")?;
        assert_eq!(value, None);

        // Test batch
        let mut batch = store.batch();
        batch.put_cf(store.get_cf("default")?, b"key2", b"value2");
        batch.put_cf(store.get_cf("default")?, b"key3", b"value3");
        store.write_batch(batch)?;

        // Test iterator
        let iter = store.iter("default")?;
        let count = iter.count();
        assert_eq!(count, 2);

        Ok(())
    }
}