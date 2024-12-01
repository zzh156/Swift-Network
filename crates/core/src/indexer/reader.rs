use super::store::{IndexStore, IndexKey, IndexValue};
use crate::protocol::{ProtocolError, ProtocolResult};
use std::sync::Arc;

/// Query options
#[derive(Debug, Clone)]
pub struct QueryOptions {
    /// Maximum results
    pub limit: Option<usize>,
    /// Cursor for pagination
    pub cursor: Option<String>,
    /// Descending order
    pub descending: bool,
}

/// Index reader
pub struct IndexReader {
    /// Index store
    store: Arc<IndexStore>,
}

impl IndexReader {
    pub fn new(store: Arc<IndexStore>) -> Self {
        Self { store }
    }

    /// Get transaction by hash
    pub async fn get_transaction(
        &self,
        hash: &TransactionDigest,
    ) -> ProtocolResult<Option<SignedTransaction>> {
        let key = IndexKey::Transaction { hash: *hash };
        match self.store.get(&key).await? {
            Some(IndexValue::Transaction(tx)) => Ok(Some(tx)),
            _ => Ok(None),
        }
    }

    /// Get transaction effects
    pub async fn get_transaction_effects(
        &self,
        hash: &TransactionDigest,
    ) -> ProtocolResult<Option<TransactionEffects>> {
        let key = IndexKey::TransactionEffects {
            transaction_hash: *hash,
        };
        match self.store.get(&key).await? {
            Some(IndexValue::TransactionEffects(effects)) => Ok(Some(effects)),
            _ => Ok(None),
        }
    }

    /// Get object by ID
    pub async fn get_object(&self, id: &ObjectID) -> ProtocolResult<Option<Object>> {
        let key = IndexKey::Object { id: *id };
        match self.store.get(&key).await? {
            Some(IndexValue::Object(object)) => Ok(Some(object)),
            _ => Ok(None),
        }
    }

    /// Get transactions by address
    pub async fn get_transactions_by_address(
        &self,
        address: &Address,
        options: QueryOptions,
    ) -> ProtocolResult<Vec<TransactionDigest>> {
        let prefix = IndexKey::address_prefix(
            address,
            AddressIndexType::Transaction,
        );
        
        let mut results = Vec::new();
        let mut iter = self.store.iter_prefix(&prefix).await?;

        if options.descending {
            iter.reverse();
        }

        if let Some(cursor) = options.cursor {
            iter.seek(&cursor)?;
        }

        while let Some((_, value)) = iter.next().await? {
            if let IndexValue::TransactionDigest(digest) = value {
                results.push(digest);
                if let Some(limit) = options.limit {
                    if results.len() >= limit {
                        break;
                    }
                }
            }
        }

        Ok(results)
    }

    /// Get objects by owner
    pub async fn get_objects_by_owner(
        &self,
        owner: &Owner,
        options: QueryOptions,
    ) -> ProtocolResult<Vec<ObjectID>> {
        let prefix = IndexKey::address_prefix(
            &owner.address(),
            AddressIndexType::Object,
        );

        let mut results = Vec::new();
        let mut iter = self.store.iter_prefix(&prefix).await?;

        if options.descending {
            iter.reverse();
        }

        if let Some(cursor) = options.cursor {
            iter.seek(&cursor)?;
        }

        while let Some((_, value)) = iter.next().await? {
            if let IndexValue::ObjectId(id) = value {
                results.push(id);
                if let Some(limit) = options.limit {
                    if results.len() >= limit {
                        break;
                    }
                }
            }
        }

        Ok(results)
    }

    /// Get events by type
    pub async fn get_events_by_type(
        &self,
        type_: &str,
        options: QueryOptions,
    ) -> ProtocolResult<Vec<Event>> {
        let prefix = IndexKey::event_prefix(type_);

        let mut results = Vec::new();
        let mut iter = self.store.iter_prefix(&prefix).await?;

        if options.descending {
            iter.reverse();
        }

        if let Some(cursor) = options.cursor {
            iter.seek(&cursor)?;
        }

        while let Some((_, value)) = iter.next().await? {
            if let IndexValue::Event(event) = value {
                results.push(event);
                if let Some(limit) = options.limit {
                    if results.len() >= limit {
                        break;
                    }
                }
            }
        }

        Ok(results)
    }
}