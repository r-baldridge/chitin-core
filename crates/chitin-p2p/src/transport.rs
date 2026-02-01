// crates/chitin-p2p/src/transport.rs
//
// TCP/QUIC transport setup for the Chitin Protocol P2P layer.

use chitin_core::ChitinError;
use libp2p::identity::Keypair;
use libp2p::Multiaddr;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::info;

use crate::behaviour::ChitinBehaviour;
use crate::SwarmHandle;

/// Configuration for the P2P transport layer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransportConfig {
    /// Multiaddr to listen on (e.g., "/ip4/0.0.0.0/tcp/9944").
    pub listen_addr: String,
    /// Whether to enable QUIC transport in addition to TCP.
    pub enable_quic: bool,
}

/// Set up the libp2p Swarm with the given configuration and keypair.
///
/// Returns a SwarmHandle that can be shared across P2P components.
pub async fn setup_transport(
    config: &TransportConfig,
    keypair: Keypair,
) -> Result<SwarmHandle, ChitinError> {
    let behaviour = ChitinBehaviour::new(&keypair)
        .map_err(|e| ChitinError::Network(format!("Failed to create behaviour: {}", e)))?;

    let mut swarm = libp2p::SwarmBuilder::with_existing_identity(keypair)
        .with_tokio()
        .with_tcp(
            libp2p::tcp::Config::default(),
            libp2p::noise::Config::new,
            libp2p::yamux::Config::default,
        )
        .map_err(|e| ChitinError::Network(format!("TCP transport error: {}", e)))?
        .with_quic()
        .with_behaviour(|_key| Ok(behaviour))
        .map_err(|e| ChitinError::Network(format!("Behaviour setup error: {}", e)))?
        .build();

    let listen_addr: Multiaddr = config
        .listen_addr
        .parse()
        .map_err(|e| ChitinError::Network(format!("Invalid multiaddr '{}': {}", config.listen_addr, e)))?;

    swarm
        .listen_on(listen_addr.clone())
        .map_err(|e| ChitinError::Network(format!("Failed to listen on {}: {}", config.listen_addr, e)))?;

    info!("P2P transport listening on {}", config.listen_addr);

    Ok(Arc::new(Mutex::new(swarm)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn setup_with_valid_addr() {
        let config = TransportConfig {
            listen_addr: "/ip4/127.0.0.1/tcp/0".to_string(),
            enable_quic: false,
        };
        let keypair = libp2p::identity::Keypair::generate_ed25519();
        let result = setup_transport(&config, keypair).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn setup_with_invalid_addr() {
        let config = TransportConfig {
            listen_addr: "not-a-multiaddr".to_string(),
            enable_quic: false,
        };
        let keypair = libp2p::identity::Keypair::generate_ed25519();
        let result = setup_transport(&config, keypair).await;
        assert!(result.is_err());
    }
}
