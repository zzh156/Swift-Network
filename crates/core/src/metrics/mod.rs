//! Metrics module for monitoring and observability.

mod metrics;

pub use metrics::{Metrics, MetricsConfig, Counter, Gauge, Histogram};

use crate::protocol::{ProtocolError, ProtocolResult};

/// Metrics error types
#[derive(Debug, thiserror::Error)]
pub enum MetricsError {
    #[error("Registration error: {0}")]
    RegistrationError(String),

    #[error("Collection error: {0}")]
    CollectionError(String),

    #[error("Export error: {0}")]
    ExportError(String),
}

pub type MetricsResult<T> = Result<T, MetricsError>;