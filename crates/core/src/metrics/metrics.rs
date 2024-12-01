use super::{MetricsError, MetricsResult};
use prometheus::{
    Counter as PrometheusCounter,
    Gauge as PrometheusGauge,
    Histogram as PrometheusHistogram,
    Registry,
    Opts,
};
use std::sync::Arc;

/// Metrics configuration
#[derive(Debug, Clone)]
pub struct MetricsConfig {
    /// Metrics namespace
    pub namespace: String,
    /// Listen address
    pub listen_address: String,
    /// Push gateway address
    pub push_gateway: Option<String>,
    /// Push interval (seconds)
    pub push_interval: u64,
}

/// Counter metric
#[derive(Clone)]
pub struct Counter {
    inner: Arc<PrometheusCounter>,
}

impl Counter {
    pub fn new(name: &str, help: &str) -> MetricsResult<Self> {
        let counter = PrometheusCounter::new(name, help)
            .map_err(|e| MetricsError::RegistrationError(e.to_string()))?;
        Ok(Self {
            inner: Arc::new(counter),
        })
    }

    pub fn inc(&self) {
        self.inner.inc();
    }

    pub fn inc_by(&self, v: f64) {
        self.inner.inc_by(v);
    }
}

/// Gauge metric
#[derive(Clone)]
pub struct Gauge {
    inner: Arc<PrometheusGauge>,
}

impl Gauge {
    pub fn new(name: &str, help: &str) -> MetricsResult<Self> {
        let gauge = PrometheusGauge::new(name, help)
            .map_err(|e| MetricsError::RegistrationError(e.to_string()))?;
        Ok(Self {
            inner: Arc::new(gauge),
        })
    }

    pub fn set(&self, v: f64) {
        self.inner.set(v);
    }

    pub fn inc(&self) {
        self.inner.inc();
    }

    pub fn dec(&self) {
        self.inner.dec();
    }
}

/// Histogram metric
#[derive(Clone)]
pub struct Histogram {
    inner: Arc<PrometheusHistogram>,
}

impl Histogram {
    pub fn new(name: &str, help: &str, buckets: Vec<f64>) -> MetricsResult<Self> {
        let histogram = PrometheusHistogram::with_opts(
            Opts::new(name, help).buckets(buckets),
        ).map_err(|e| MetricsError::RegistrationError(e.to_string()))?;
        Ok(Self {
            inner: Arc::new(histogram),
        })
    }

    pub fn observe(&self, v: f64) {
        self.inner.observe(v);
    }
}

/// Metrics system
pub struct Metrics {
    /// Configuration
    config: MetricsConfig,
    /// Registry
    registry: Registry,
    /// Transaction metrics
    pub transactions: TransactionMetrics,
    /// Consensus metrics
    pub consensus: ConsensusMetrics,
    /// Network metrics
    pub network: NetworkMetrics,
    /// Storage metrics
    pub storage: StorageMetrics,
}

impl Metrics {
    /// Create new metrics system
    pub fn new(config: MetricsConfig) -> MetricsResult<Self> {
        let registry = Registry::new();

        let transactions = TransactionMetrics::new(&registry)?;
        let consensus = ConsensusMetrics::new(&registry)?;
        let network = NetworkMetrics::new(&registry)?;
        let storage = StorageMetrics::new(&registry)?;

        Ok(Self {
            config,
            registry,
            transactions,
            consensus,
            network,
            storage,
        })
    }

    /// Start metrics server
    pub async fn start_server(&self) -> MetricsResult<()> {
        use warp::Filter;

        let metrics = warp::path!("metrics")
            .map(move || {
                use prometheus::Encoder;
                let encoder = prometheus::TextEncoder::new();
                let mut buffer = Vec::new();
                encoder.encode(&self.registry.gather(), &mut buffer)
                    .expect("Failed to encode metrics");
                String::from_utf8(buffer).expect("Failed to convert metrics to string")
            });

        let addr = self.config.listen_address.parse()
            .map_err(|e| MetricsError::ExportError(e.to_string()))?;

        tokio::spawn(warp::serve(metrics).run(addr));

        Ok(())
    }

    /// Start push gateway client
    pub async fn start_push_client(&self) -> MetricsResult<()> {
        if let Some(gateway) = &self.config.push_gateway {
            let gateway = gateway.clone();
            let registry = self.registry.clone();
            let interval = self.config.push_interval;

            tokio::spawn(async move {
                let client = reqwest::Client::new();
                loop {
                    tokio::time::sleep(tokio::time::Duration::from_secs(interval)).await;
                    
                    let metrics = {
                        use prometheus::Encoder;
                        let encoder = prometheus::TextEncoder::new();
                        let mut buffer = Vec::new();
                        encoder.encode(&registry.gather(), &mut buffer)
                            .expect("Failed to encode metrics");
                        buffer
                    };

                    if let Err(e) = client.post(&gateway)
                        .body(metrics)
                        .send()
                        .await
                    {
                        log::error!("Failed to push metrics: {}", e);
                    }
                }
            });
        }

        Ok(())
    }
}

/// Transaction metrics
#[derive(Clone)]
pub struct TransactionMetrics {
    pub total_transactions: Counter,
    pub pending_transactions: Gauge,
    pub transaction_latency: Histogram,
}

impl TransactionMetrics {
    fn new(registry: &Registry) -> MetricsResult<Self> {
        let total_transactions = Counter::new("total_transactions", "Total transactions processed")?;
        let pending_transactions = Gauge::new("pending_transactions", "Pending transactions")?;
        let transaction_latency = Histogram::new(
            "transaction_latency",
            "Transaction processing latency",
            vec![0.001, 0.01, 0.1, 1.0, 10.0],
        )?;

        registry.register(Box::new(total_transactions.inner.clone()))
            .map_err(|e| MetricsError::RegistrationError(e.to_string()))?;
        registry.register(Box::new(pending_transactions.inner.clone()))
            .map_err(|e| MetricsError::RegistrationError(e.to_string()))?;
        registry.register(Box::new(transaction_latency.inner.clone()))
            .map_err(|e| MetricsError::RegistrationError(e.to_string()))?;

        Ok(Self {
            total_transactions,
            pending_transactions,
            transaction_latency,
        })
    }
}

/// Consensus metrics
#[derive(Clone)]
pub struct ConsensusMetrics {
    pub consensus_rounds: Counter,
    pub active_validators: Gauge,
    pub consensus_latency: Histogram,
}

impl ConsensusMetrics {
    fn new(registry: &Registry) -> MetricsResult<Self> {
        let consensus_rounds = Counter::new("consensus_rounds", "Total consensus rounds")?;
        let active_validators = Gauge::new("active_validators", "Active validators")?;
        let consensus_latency = Histogram::new(
            "consensus_latency",
            "Consensus round latency",
            vec![0.1, 1.0, 10.0, 100.0],
        )?;

        registry.register(Box::new(consensus_rounds.inner.clone()))
            .map_err(|e| MetricsError::RegistrationError(e.to_string()))?;
        registry.register(Box::new(active_validators.inner.clone()))
            .map_err(|e| MetricsError::RegistrationError(e.to_string()))?;
        registry.register(Box::new(consensus_latency.inner.clone()))
            .map_err(|e| MetricsError::RegistrationError(e.to_string()))?;

        Ok(Self {
            consensus_rounds,
            active_validators,
            consensus_latency,
        })
    }
}

/// Network metrics
#[derive(Clone)]
pub struct NetworkMetrics {
    pub connected_peers: Gauge,
    pub network_messages: Counter,
    pub message_latency: Histogram,
}

impl NetworkMetrics {
    fn new(registry: &Registry) -> MetricsResult<Self> {
        let connected_peers = Gauge::new("connected_peers", "Connected peers")?;
        let network_messages = Counter::new("network_messages", "Total network messages")?;
        let message_latency = Histogram::new(
            "message_latency",
            "Network message latency",
            vec![0.001, 0.01, 0.1, 1.0],
        )?;

        registry.register(Box::new(connected_peers.inner.clone()))
            .map_err(|e| MetricsError::RegistrationError(e.to_string()))?;
        registry.register(Box::new(network_messages.inner.clone()))
            .map_err(|e| MetricsError::RegistrationError(e.to_string()))?;
        registry.register(Box::new(message_latency.inner.clone()))
            .map_err(|e| MetricsError::RegistrationError(e.to_string()))?;

        Ok(Self {
            connected_peers,
            network_messages,
            message_latency,
        })
    }
}

/// Storage metrics
#[derive(Clone)]
pub struct StorageMetrics {
    pub total_objects: Gauge,
    pub storage_operations: Counter,
    pub operation_latency: Histogram,
}

impl StorageMetrics {
    fn new(registry: &Registry) -> MetricsResult<Self> {
        let total_objects = Gauge::new("total_objects", "Total objects in storage")?;
        let storage_operations = Counter::new("storage_operations", "Total storage operations")?;
        let operation_latency = Histogram::new(
            "operation_latency",
            "Storage operation latency",
            vec![0.001, 0.01, 0.1, 1.0],
        )?;

        registry.register(Box::new(total_objects.inner.clone()))
            .map_err(|e| MetricsError::RegistrationError(e.to_string()))?;
        registry.register(Box::new(storage_operations.inner.clone()))
            .map_err(|e| MetricsError::RegistrationError(e.to_string()))?;
        registry.register(Box::new(operation_latency.inner.clone()))
            .map_err(|e| MetricsError::RegistrationError(e.to_string()))?;

        Ok(Self {
            total_objects,
            storage_operations,
            operation_latency,
        })
    }
}