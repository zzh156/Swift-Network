// storage/event_store.rs
use super::rocks_store::RocksStore;
use crate::protocol::{ProtocolError, ProtocolResult};
use serde::{Serialize, Deserialize};
use std::sync::Arc;
use chrono::{DateTime, Utc};

/// Event type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventType {
    /// Transaction related
    Transaction(TransactionEvent),
    /// Object related
    Object(ObjectEvent),
    /// System related
    System(SystemEvent),
    /// Custom event
    Custom(String),
}

/// Transaction event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransactionEvent {
    /// Transaction submitted
    Submitted {
        tx_digest: String,
        sender: String,
    },
    /// Transaction executed
    Executed {
        tx_digest: String,
        success: bool,
        error: Option<String>,
    },
    /// Transaction certified
    Certified {
        tx_digest: String,
        certificate: String,
    },
}

/// Object event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ObjectEvent {
    /// Object created
    Created {
        object_id: String,
        owner: String,
        type_: String,
    },
    /// Object modified
    Modified {
        object_id: String,
        version: u64,
        changes: Vec<String>,
    },
    /// Object deleted
    Deleted {
        object_id: String,
        version: u64,
    },
}

/// System event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SystemEvent {
    /// Epoch advanced
    EpochAdvanced {
        old_epoch: u64,
        new_epoch: u64,
    },
    /// Checkpoint created
    CheckpointCreated {
        sequence: u64,
        root_hash: String,
    },
    /// Validator set changed
    ValidatorSetChanged {
        added: Vec<String>,
        removed: Vec<String>,
    },
}

/// Event data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    /// Event ID
    pub id: String,
    /// Event type
    pub type_: EventType,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Additional metadata
    pub metadata: Option<serde_json::Value>,
}

/// Event filter
#[derive(Debug, Clone)]
pub struct EventFilter {
    /// Event types to include
    pub types: Option<Vec<EventType>>,
    /// Start time
    pub start_time: Option<DateTime<Utc>>,
    /// End time
    pub end_time: Option<DateTime<Utc>>,
    /// Maximum events to return
    pub limit: Option<usize>,
}

/// Event store implementation
pub struct EventStore {
    /// RocksDB store
    rocks: Arc<RocksStore>,
    /// Column family for events
    events_cf: String,
    /// Column family for indexes
    indexes_cf: String,
}

impl EventStore {
    pub fn new(rocks: Arc<RocksStore>) -> Self {
        Self {
            rocks,
            events_cf: "events".to_string(),
            indexes_cf: "event_indexes".to_string(),
        }
    }

    /// Emit new event
    pub fn emit_event(&self, event: Event) -> ProtocolResult<()> {
        // Generate event ID if not present
        let event = if event.id.is_empty() {
            Event {
                id: format!("evt_{}", uuid::Uuid::new_v4()),
                ..event
            }
        } else {
            event
        };

        // Serialize event
        let key = event.id.as_bytes();
        let value = bincode::serialize(&event)?;

        // Create batch
        let batch = self.rocks.batch();
        
        // Write event
        batch.put(&self.events_cf, key, &value)?;

        // Update indexes
        self.update_indexes(&batch, &event)?;

        // Commit batch
        batch.write()?;

        Ok(())
    }

    /// Get events by filter
    pub fn get_events(&self, filter: &EventFilter) -> ProtocolResult<Vec<Event>> {
        let mut events = Vec::new();
        let iter = self.rocks.iter(&self.events_cf)?;

        for item in iter {
            let (_, value_bytes) = item?;
            let event: Event = bincode::deserialize(&value_bytes)?;

            // Apply filters
            if self.matches_filter(&event, filter) {
                events.push(event);
            }

            // Check limit
            if let Some(limit) = filter.limit {
                if events.len() >= limit {
                    break;
                }
            }
        }

        // Sort by timestamp
        events.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

        Ok(events)
    }

    /// Get event by ID
    pub fn get_event(&self, id: &str) -> ProtocolResult<Option<Event>> {
        let key = id.as_bytes();
        let value = self.rocks.get(&self.events_cf, key)?;

        match value {
            Some(bytes) => {
                let event: Event = bincode::deserialize(&bytes)?;
                Ok(Some(event))
            }
            None => Ok(None),
        }
    }

    /// Update event indexes
    fn update_indexes(&self, batch: &rocksdb::WriteBatch, event: &Event) -> ProtocolResult<()> {
        // Index by type
        let type_key = format!("type:{}:{}", self.get_type_key(&event.type_), event.id);
        batch.put(&self.indexes_cf, type_key.as_bytes(), &[])?;

        // Index by timestamp
        let time_key = format!("time:{}:{}", event.timestamp.timestamp(), event.id);
        batch.put(&self.indexes_cf, time_key.as_bytes(), &[])?;

        Ok(())
    }

    /// Get type key for indexing
    fn get_type_key(&self, type_: &EventType) -> String {
        match type_ {
            EventType::Transaction(_) => "tx",
            EventType::Object(_) => "obj",
            EventType::System(_) => "sys",
            EventType::Custom(_) => "custom",
        }.to_string()
    }

    /// Check if event matches filter
    fn matches_filter(&self, event: &Event, filter: &EventFilter) -> bool {
        // Check type
        if let Some(types) = &filter.types {
            if !types.iter().any(|t| std::mem::discriminant(t) == std::mem::discriminant(&event.type_)) {
                return false;
            }
        }

        // Check time range
        if let Some(start) = filter.start_time {
            if event.timestamp < start {
                return false;
            }
        }

        if let Some(end) = filter.end_time {
            if event.timestamp > end {
                return false;
            }
        }

        true
    }

    /// Prune old events
    pub fn prune_events(&self, before: DateTime<Utc>) -> ProtocolResult<u64> {
        let mut count = 0;
        let batch = self.rocks.batch();

        let iter = self.rocks.iter(&self.events_cf)?;
        for item in iter {
            let (key_bytes, value_bytes) = item?;
            let event: Event = bincode::deserialize(&value_bytes)?;

            if event.timestamp < before {
                batch.delete(&self.events_cf, &key_bytes)?;
                count += 1;
            }
        }

        batch.write()?;
        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_event_store() -> ProtocolResult<()> {
        let temp_dir = TempDir::new()?;
        let rocks = Arc::new(RocksStore::new(&RocksConfig {
            path: temp_dir.path().to_str().unwrap().to_string(),
            ..Default::default()
        })?);

        let store = EventStore::new(rocks);

        // Test emit event
        let event = Event {
            id: String::new(),
            type_: EventType::System(SystemEvent::EpochAdvanced {
                old_epoch: 1,
                new_epoch: 2,
            }),
            timestamp: Utc::now(),
            metadata: None,
        };

        store.emit_event(event.clone())?;

        // Test get events
        let filter = EventFilter {
            types: Some(vec![EventType::System(SystemEvent::EpochAdvanced {
                old_epoch: 0,
                new_epoch: 0,
            })]),
            start_time: None,
            end_time: None,
            limit: None,
        };

        let events = store.get_events(&filter)?;
        assert_eq!(events.len(), 1);

        Ok(())
    }
}