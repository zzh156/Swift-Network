// telemetry/metrics.rs
use prometheus::{
    Counter, Gauge, Histogram, HistogramOpts, IntCounter, IntCounterVec, 
    IntGauge, IntGaugeVec, Registry,
};
use lazy_static::lazy_static;
use std::sync::Arc;

/// 系统指标收集器
pub struct Metrics {
    registry: Registry,
    
    // 交易相关指标
    pub tx_processed_count: IntCounter,
    pub tx_processing_time: Histogram,
    pub tx_in_mempool: IntGauge,
    pub tx_by_type: IntCounterVec,
    
    // 共识相关指标
    pub consensus_rounds: IntCounter,
    pub consensus_validators: IntGauge,
    pub consensus_latency: Histogram,
    
    // 网络相关指标
    pub network_peers: IntGauge,
    pub network_messages: IntCounterVec,
    pub network_bandwidth: IntGaugeVec,
    
    // 存储相关指标
    pub storage_objects: IntGauge,
    pub storage_size: IntGaugeVec,
    pub storage_operations: IntCounterVec,
}

impl Metrics {
    pub fn new(endpoint: &str) -> Self {
        let registry = Registry::new();

        // 创建交易指标
        let tx_processed_count = IntCounter::new(
            "sui_tx_processed_total",
            "Total number of processed transactions"
        ).unwrap();

        let tx_processing_time = Histogram::with_opts(
            HistogramOpts::new(
                "sui_tx_processing_duration",
                "Transaction processing duration in seconds"
            )
        ).unwrap();

        let tx_in_mempool = IntGauge::new(
            "sui_tx_in_mempool",
            "Number of transactions currently in mempool"
        ).unwrap();

        let tx_by_type = IntCounterVec::new(
            "sui_tx_by_type_total",
            "Transactions by type",
            &["type"]
        ).unwrap();

        // 创建共识指标
        let consensus_rounds = IntCounter::new(
            "sui_consensus_rounds_total",
            "Total number of consensus rounds"
        ).unwrap();

        let consensus_validators = IntGauge::new(
            "sui_consensus_validators",
            "Number of active validators"
        ).unwrap();

        let consensus_latency = Histogram::with_opts(
            HistogramOpts::new(
                "sui_consensus_latency",
                "Consensus round latency in seconds"
            )
        ).unwrap();

        // 创建网络指标
        let network_peers = IntGauge::new(
            "sui_network_peers",
            "Number of connected peers"
        ).unwrap();

        let network_messages = IntCounterVec::new(
            "sui_network_messages_total",
            "Network messages by type",
            &["type"]
        ).unwrap();

        let network_bandwidth = IntGaugeVec::new(
            "sui_network_bandwidth_bytes",
            "Network bandwidth usage in bytes",
            &["direction"]
        ).unwrap();

        // 创建存储指标
        let storage_objects = IntGauge::new(
            "sui_storage_objects",
            "Total number of objects in storage"
        ).unwrap();

        let storage_size = IntGaugeVec::new(
            "sui_storage_size_bytes",
            "Storage size in bytes",
            &["type"]
        ).unwrap();

        let storage_operations = IntCounterVec::new(
            "sui_storage_operations_total",
            "Storage operations by type",
            &["operation"]
        ).unwrap();

        // 注册所有指标
        registry.register(Box::new(tx_processed_count.clone())).unwrap();
        registry.register(Box::new(tx_processing_time.clone())).unwrap();
        registry.register(Box::new(tx_in_mempool.clone())).unwrap();
        registry.register(Box::new(tx_by_type.clone())).unwrap();
        registry.register(Box::new(consensus_rounds.clone())).unwrap();
        registry.register(Box::new(consensus_validators.clone())).unwrap();
        registry.register(Box::new(consensus_latency.clone())).unwrap();
        registry.register(Box::new(network_peers.clone())).unwrap();
        registry.register(Box::new(network_messages.clone())).unwrap();
        registry.register(Box::new(network_bandwidth.clone())).unwrap();
        registry.register(Box::new(storage_objects.clone())).unwrap();
        registry.register(Box::new(storage_size.clone())).unwrap();
        registry.register(Box::new(storage_operations.clone())).unwrap();

        Self {
            registry,
            tx_processed_count,
            tx_processing_time,
            tx_in_mempool,
            tx_by_type,
            consensus_rounds,
            consensus_validators,
            consensus_latency,
            network_peers,
            network_messages,
            network_bandwidth,
            storage_objects,
            storage_size,
            storage_operations,
        }
    }

    // 交易相关方法
    pub fn record_tx_processed(&self) {
        self.tx_processed_count.inc();
    }

    pub fn observe_tx_processing_time(&self, duration_secs: f64) {
        self.tx_processing_time.observe(duration_secs);
    }

    pub fn set_mempool_size(&self, size: i64) {
        self.tx_in_mempool.set(size);
    }

    // 共识相关方法
    pub fn record_consensus_round(&self) {
        self.consensus_rounds.inc();
    }

    pub fn set_validator_count(&self, count: i64) {
        self.consensus_validators.set(count);
    }

    pub fn observe_consensus_latency(&self, duration_secs: f64) {
        self.consensus_latency.observe(duration_secs);
    }

    // 网络相关方法
    pub fn set_peer_count(&self, count: i64) {
        self.network_peers.set(count);
    }

    pub fn record_network_message(&self, message_type: &str) {
        self.network_messages.with_label_values(&[message_type]).inc();
    }

    pub fn set_bandwidth_usage(&self, direction: &str, bytes: i64) {
        self.network_bandwidth.with_label_values(&[direction]).set(bytes);
    }

    // 存储相关方法
    pub fn set_object_count(&self, count: i64) {
        self.storage_objects.set(count);
    }

    pub fn set_storage_size(&self, type_: &str, bytes: i64) {
        self.storage_size.with_label_values(&[type_]).set(bytes);
    }

    pub fn record_storage_operation(&self, operation: &str) {
        self.storage_operations.with_label_values(&[operation]).inc();
    }

    // 获取所有指标的当前快照
    pub fn gather(&self) -> Vec<prometheus::proto::MetricFamily> {
        self.registry.gather()
    }
}