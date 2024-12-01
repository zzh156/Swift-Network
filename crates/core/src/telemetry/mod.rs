//! Telemetry module for logging, tracing, and metrics collection.

mod logging;
mod tracing;
mod metrics;

pub use logging::{Logger, LogLevel};
pub use tracing::{Tracer, Span, SpanContext};
pub use metrics::{Metrics, Counter, Gauge, Histogram};

use std::sync::Arc;

/// Telemetry configuration
#[derive(Debug, Clone)]
pub struct TelemetryConfig {
    /// Log level
    pub log_level: LogLevel,
    /// Enable tracing
    pub enable_tracing: bool,
    /// Metrics endpoint
    pub metrics_endpoint: String,
}

/// Telemetry system
pub struct Telemetry {
    config: TelemetryConfig,
    logger: Arc<Logger>,
    tracer: Arc<Tracer>,
    metrics: Arc<Metrics>,
}

impl Telemetry {
    pub fn new(config: TelemetryConfig) -> Self {
        let logger = Arc::new(Logger::new(config.log_level));
        let tracer = Arc::new(Tracer::new(config.enable_tracing));
        let metrics = Arc::new(Metrics::new(&config.metrics_endpoint));

        Self {
            config,
            logger,
            tracer,
            metrics,
        }
    }

    pub fn logger(&self) -> Arc<Logger> {
        self.logger.clone()
    }

    pub fn tracer(&self) -> Arc<Tracer> {
        self.tracer.clone()
    }

    pub fn metrics(&self) -> Arc<Metrics> {
        self.metrics.clone()
    }
}