//! Indexer module for blockchain data indexing and querying.

mod builder;
mod reader;
mod store;

pub use builder::{IndexBuilder, IndexConfig};
pub use reader::{IndexReader, QueryOptions};
pub use store::{IndexStore, IndexKey, IndexValue};

use crate::protocol::{ProtocolError, ProtocolResult};
use std::sync::Arc;

/// Indexer configuration
#[derive(Debug, Clone)]
pub struct IndexerConfig {
    /// Builder configuration
    pub builder: IndexConfig,
    /// Store configuration
    pub store: StoreConfig,
}

/// Indexer manager
pub struct Indexer {
    /// Index builder
    builder: Arc<IndexBuilder>,
    /// Index reader
    reader: Arc<IndexReader>,
    /// Index store
    store: Arc<IndexStore>,
}

impl Indexer {
    pub fn new(config: IndexerConfig) -> ProtocolResult<Self> {
        let store = Arc::new(IndexStore::new(config.store)?);
        let builder = Arc::new(IndexBuilder::new(config.builder, store.clone())?);
        let reader = Arc::new(IndexReader::new(store.clone()));

        Ok(Self {
            builder,
            reader,
            store,
        })
    }

    pub fn builder(&self) -> Arc<IndexBuilder> {
        self.builder.clone()
    }

    pub fn reader(&self) -> Arc<IndexReader> {
        self.reader.clone()
    }

    pub fn store(&self) -> Arc<IndexStore> {
        self.store.clone()
    }
}