use super::{StateError, StateResult, StateStore};
use std::sync::Arc;
use tokio::time::{Duration, Interval};

/// Prune configuration
#[derive(Debug, Clone)]
pub struct PruneConfig {
    /// Minimum checkpoints to keep
    pub min_checkpoints: u64,
    /// Maximum checkpoints to keep
    pub max_checkpoints: u64,
    /// Prune interval
    pub prune_interval: Duration,
}

impl Default for PruneConfig {
    fn default() -> Self {
        Self {
            min_checkpoints: 1000,
            max_checkpoints: 10000,
            prune_interval: Duration::from_secs(3600),
        }
    }
}

/// State pruner
pub struct StatePruner {
    /// Configuration
    config: PruneConfig,
    /// State store
    store: Arc<StateStore>,
    /// Prune interval
    interval: Interval,
}

impl StatePruner {
    /// Create new pruner
    pub fn new(config: PruneConfig, store: Arc<StateStore>) -> Self {
        Self {
            interval: tokio::time::interval(config.prune_interval),
            config,
            store,
        }
    }

    /// Start pruning
    pub async fn start(&mut self) {
        loop {
            self.interval.tick().await;
            if let Err(e) = self.prune().await {
                log::error!("Pruning failed: {}", e);
            }
        }
    }

    /// Prune old state
    async fn prune(&self) -> StateResult<()> {
        // Get latest checkpoint
        let latest = match self.store.get_latest_checkpoint().await? {
            Some(checkpoint) => checkpoint,
            None => return Ok(()),
        };

        // Calculate pruning target
        let target = if latest.sequence > self.config.max_checkpoints {
            latest.sequence - self.config.max_checkpoints
        } else {
            return Ok(());
        };

        // Don't prune below minimum
        if target < self.config.min_checkpoints {
            return Ok(());
        }

        // Prune old checkpoints
        for sequence in 0..target {
            self.store.delete_checkpoint(sequence).await?;
        }

        // Prune old state
        self.store.prune_state(target).await?;

        Ok(())
    }
}