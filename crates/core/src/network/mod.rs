//! Network module for P2P communication.

mod p2p;

pub use p2p::{NetworkService, NetworkConfig, NetworkMessage, PeerInfo};

use crate::protocol::{ProtocolError, ProtocolResult};
use std::sync::Arc;

/// Network error types
#[derive(Debug, thiserror::Error)]
pub enum NetworkError {
    #[error("Connection error: {0}")]
    ConnectionError(String),

    #[error("Peer error: {0}")]
    PeerError(String),

    #[error("Message error: {0}")]
    MessageError(String),

    #[error("Protocol error: {0}")]
    ProtocolError(String),
}

pub type NetworkResult<T> = Result<T, NetworkError>;

/// Network event
#[derive(Debug, Clone)]
pub enum NetworkEvent {
    /// New peer connected
    PeerConnected(PeerInfo),
    /// Peer disconnected
    PeerDisconnected(PeerInfo),
    /// Message received
    MessageReceived {
        peer: PeerInfo,
        message: NetworkMessage,
    },
}

/// Network event handler
#[async_trait::async_trait]
pub trait NetworkEventHandler: Send + Sync {
    /// Handle network event
    async fn handle_event(&self, event: NetworkEvent) -> NetworkResult<()>;
}