use super::{NetworkError, NetworkEvent, NetworkEventHandler, NetworkResult};
use crate::protocol::{ProtocolError, ProtocolResult};
use libp2p::{
    core::{muxing::StreamMuxerBox, transport::Boxed},
    identity, mplex, noise,
    swarm::{NetworkBehaviour, SwarmBuilder, SwarmEvent},
    tcp, yamux, Multiaddr, PeerId, Swarm, Transport,
};
use std::sync::Arc;
use tokio::sync::mpsc;

/// Network configuration
#[derive(Debug, Clone)]
pub struct NetworkConfig {
    /// Listen addresses
    pub listen_addresses: Vec<Multiaddr>,
    /// Bootstrap peers
    pub bootstrap_peers: Vec<Multiaddr>,
    /// Maximum peers
    pub max_peers: usize,
    /// Connection timeout
    pub connection_timeout: std::time::Duration,
    /// Protocol version
    pub protocol_version: String,
}

/// Peer information
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PeerInfo {
    /// Peer ID
    pub peer_id: PeerId,
    /// Peer address
    pub address: Multiaddr,
    /// Protocol version
    pub protocol_version: String,
}

/// Network message
#[derive(Debug, Clone)]
pub enum NetworkMessage {
    /// Transaction message
    Transaction(TransactionMessage),
    /// Consensus message
    Consensus(ConsensusMessage),
    /// State sync message
    StateSync(StateSyncMessage),
}

/// Network service
pub struct NetworkService {
    /// Configuration
    config: NetworkConfig,
    /// Swarm
    swarm: Swarm<NetworkBehaviour>,
    /// Event sender
    event_sender: mpsc::Sender<NetworkEvent>,
    /// Event handler
    event_handler: Arc<dyn NetworkEventHandler>,
}

impl NetworkService {
    /// Create new network service
    pub async fn new(
        config: NetworkConfig,
        event_handler: Arc<dyn NetworkEventHandler>,
    ) -> NetworkResult<Self> {
        // Create identity
        let identity = identity::Keypair::generate_ed25519();
        let peer_id = PeerId::from(identity.public());

        // Create transport
        let transport = build_transport(identity.clone())?;

        // Create behaviour
        let behaviour = build_behaviour(config.clone())?;

        // Create swarm
        let swarm = SwarmBuilder::new(transport, behaviour, peer_id)
            .executor(Box::new(|fut| {
                tokio::spawn(fut);
            }))
            .build();

        // Create event channel
        let (event_sender, mut event_receiver) = mpsc::channel(1000);

        // Create service
        let mut service = Self {
            config,
            swarm,
            event_sender,
            event_handler,
        };

        // Start event loop
        tokio::spawn(async move {
            while let Some(event) = event_receiver.recv().await {
                if let Err(e) = event_handler.handle_event(event).await {
                    log::error!("Failed to handle network event: {}", e);
                }
            }
        });

        // Start listening
        service.start_listening().await?;

        // Connect to bootstrap peers
        service.connect_bootstrap_peers().await?;

        Ok(service)
    }

    /// Start listening
    async fn start_listening(&mut self) -> NetworkResult<()> {
        for addr in &self.config.listen_addresses {
            self.swarm.listen_on(addr.clone())
                .map_err(|e| NetworkError::ConnectionError(e.to_string()))?;
        }
        Ok(())
    }

    /// Connect to bootstrap peers
    async fn connect_bootstrap_peers(&mut self) -> NetworkResult<()> {
        for addr in &self.config.bootstrap_peers {
            self.connect_peer(addr.clone()).await?;
        }
        Ok(())
    }

    /// Connect to peer
    pub async fn connect_peer(&mut self, addr: Multiaddr) -> NetworkResult<()> {
        self.swarm.dial(addr.clone())
            .map_err(|e| NetworkError::ConnectionError(e.to_string()))?;
        Ok(())
    }

    /// Broadcast message
    pub async fn broadcast(&mut self, message: NetworkMessage) -> NetworkResult<()> {
        // Get connected peers
        let peers: Vec<_> = self.swarm.connected_peers().cloned().collect();

        // Send message to all peers
        for peer_id in peers {
            self.send_message(peer_id, message.clone()).await?;
        }

        Ok(())
    }

    /// Send message to peer
    pub async fn send_message(
        &mut self,
        peer_id: PeerId,
        message: NetworkMessage,
    ) -> NetworkResult<()> {
        // Serialize message
        let data = bincode::serialize(&message)
            .map_err(|e| NetworkError::MessageError(e.to_string()))?;

        // Send message
        self.swarm.behaviour_mut().send_message(peer_id, data)
            .map_err(|e| NetworkError::MessageError(e.to_string()))?;

        Ok(())
    }

    /// Run network service
    pub async fn run(&mut self) -> NetworkResult<()> {
        loop {
            match self.swarm.next_event().await {
                SwarmEvent::NewListenAddr { address, .. } => {
                    log::info!("Listening on {}", address);
                }
                SwarmEvent::ConnectionEstablished { peer_id, endpoint, .. } => {
                    let peer_info = PeerInfo {
                        peer_id,
                        address: endpoint.get_remote_address().clone(),
                        protocol_version: self.config.protocol_version.clone(),
                    };
                    self.event_sender.send(NetworkEvent::PeerConnected(peer_info)).await
                        .map_err(|e| NetworkError::MessageError(e.to_string()))?;
                }
                SwarmEvent::ConnectionClosed { peer_id, endpoint, .. } => {
                    let peer_info = PeerInfo {
                        peer_id,
                        address: endpoint.get_remote_address().clone(),
                        protocol_version: self.config.protocol_version.clone(),
                    };
                    self.event_sender.send(NetworkEvent::PeerDisconnected(peer_info)).await
                        .map_err(|e| NetworkError::MessageError(e.to_string()))?;
                }
                SwarmEvent::Behaviour(event) => {
                    self.handle_behaviour_event(event).await?;
                }
                _ => {}
            }
        }
    }

    /// Handle behaviour event
    async fn handle_behaviour_event(&mut self, event: BehaviourEvent) -> NetworkResult<()> {
        match event {
            BehaviourEvent::Message { peer_id, data } => {
                // Deserialize message
                let message: NetworkMessage = bincode::deserialize(&data)
                    .map_err(|e| NetworkError::MessageError(e.to_string()))?;

                // Create peer info
                let peer_info = PeerInfo {
                    peer_id,
                    address: self.swarm.behaviour().get_peer_address(&peer_id)
                        .ok_or_else(|| NetworkError::PeerError("Peer not found".into()))?.clone(),
                    protocol_version: self.config.protocol_version.clone(),
                };

                // Send event
                self.event_sender.send(NetworkEvent::MessageReceived {
                    peer: peer_info,
                    message,
                }).await
                    .map_err(|e| NetworkError::MessageError(e.to_string()))?;
            }
        }
        Ok(())
    }
}

/// Build transport
fn build_transport(
    identity: identity::Keypair,
) -> NetworkResult<Boxed<(PeerId, StreamMuxerBox)>> {
    let transport = tcp::TcpConfig::new()
        .nodelay(true)
        .upgrade(libp2p::core::upgrade::Version::V1)
        .authenticate(noise::NoiseConfig::xx(identity).into_authenticated())
        .multiplex(yamux::YamuxConfig::default())
        .boxed();
    Ok(transport)
}

/// Build network behaviour
fn build_behaviour(config: NetworkConfig) -> NetworkResult<NetworkBehaviour> {
    Ok(NetworkBehaviour::new(config))
}