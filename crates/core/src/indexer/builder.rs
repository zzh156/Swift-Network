use super::store::{IndexStore, IndexKey, IndexValue};
use crate::protocol::{ProtocolError, ProtocolResult};
use std::sync::Arc;

/// Index builder configuration
#[derive(Debug, Clone)]
pub struct IndexConfig {
    /// Maximum batch size
    pub max_batch_size: usize,
    /// Index types to build
    pub index_types: Vec<IndexType>,
}

/// Index types
#[derive(Debug, Clone, PartialEq)]
pub enum IndexType {
    /// Transaction index
    Transaction,
    /// Object index
    Object,
    /// Event index
    Event,
    /// Address index
    Address,
}

/// Index builder
pub struct IndexBuilder {
    /// Configuration
    config: IndexConfig,
    /// Index store
    store: Arc<IndexStore>,
    /// Current batch
    batch: Vec<(IndexKey, IndexValue)>,
}

impl IndexBuilder {
    pub fn new(config: IndexConfig, store: Arc<IndexStore>) -> ProtocolResult<Self> {
        Ok(Self {
            config,
            store,
            batch: Vec::new(),
        })
    }

    /// Index transaction
    pub async fn index_transaction(
        &mut self,
        transaction: &SignedTransaction,
        effects: &TransactionEffects,
    ) -> ProtocolResult<()> {
        if !self.config.index_types.contains(&IndexType::Transaction) {
            return Ok(());
        }

        // Index transaction by hash
        let key = IndexKey::Transaction {
            hash: transaction.digest(),
        };
        let value = IndexValue::Transaction(transaction.clone());
        self.add_to_batch(key, value)?;

        // Index transaction by sender
        let key = IndexKey::Address {
            address: transaction.sender(),
            type_: AddressIndexType::Transaction,
            timestamp: transaction.timestamp(),
        };
        let value = IndexValue::TransactionDigest(transaction.digest());
        self.add_to_batch(key, value)?;

        // Index effects
        let key = IndexKey::TransactionEffects {
            transaction_hash: transaction.digest(),
        };
        let value = IndexValue::TransactionEffects(effects.clone());
        self.add_to_batch(key, value)?;

        Ok(())
    }

    /// Index object
    pub async fn index_object(
        &mut self,
        object: &Object,
        owner: &Owner,
    ) -> ProtocolResult<()> {
        if !self.config.index_types.contains(&IndexType::Object) {
            return Ok(());
        }

        // Index object by ID
        let key = IndexKey::Object {
            id: object.id(),
        };
        let value = IndexValue::Object(object.clone());
        self.add_to_batch(key, value)?;

        // Index object by owner
        let key = IndexKey::Address {
            address: owner.address(),
            type_: AddressIndexType::Object,
            timestamp: object.version().value(),
        };
        let value = IndexValue::ObjectId(object.id());
        self.add_to_batch(key, value)?;

        Ok(())
    }

    /// Index event
    pub async fn index_event(&mut self, event: &Event) -> ProtocolResult<()> {
        if !self.config.index_types.contains(&IndexType::Event) {
            return Ok(());
        }

        // Index event by type
        let key = IndexKey::Event {
            type_: event.type_str().to_string(),
            timestamp: event.timestamp(),
        };
        let value = IndexValue::Event(event.clone());
        self.add_to_batch(key, value)?;

        Ok(())
    }

    /// Add to batch
    fn add_to_batch(&mut self, key: IndexKey, value: IndexValue) -> ProtocolResult<()> {
        self.batch.push((key, value));

        if self.batch.len() >= self.config.max_batch_size {
            self.flush_batch().await?;
        }

        Ok(())
    }

    /// Flush batch
    async fn flush_batch(&mut self) -> ProtocolResult<()> {
        if self.batch.is_empty() {
            return Ok(());
        }

        // Write batch to store
        self.store.put_batch(std::mem::take(&mut self.batch)).await?;

        Ok(())
    }
}