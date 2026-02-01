// crates/chitin-p2p/src/discovery.rs
//
// mDNS + Kademlia DHT peer discovery for the Chitin Protocol.

use chitin_core::ChitinError;
use libp2p::Multiaddr;
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::SwarmHandle;

/// Configuration for peer discovery mechanisms.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryConfig {
    /// Enable mDNS for local network peer discovery.
    pub enable_mdns: bool,
    /// Bootstrap peers to connect to on startup (multiaddrs).
    pub bootstrap_peers: Vec<String>,
}

/// Start peer discovery using Kademlia DHT bootstrap peers.
///
/// Adds bootstrap peers to Kademlia's routing table and triggers
/// a bootstrap query. mDNS auto-starts as part of the behaviour.
pub async fn start_discovery(
    swarm: &SwarmHandle,
    config: &DiscoveryConfig,
) -> Result<(), ChitinError> {
    let mut swarm_guard = swarm.lock().await;

    // Add bootstrap peers to Kademlia
    for peer_addr_str in &config.bootstrap_peers {
        let addr: Multiaddr = peer_addr_str
            .parse()
            .map_err(|e| ChitinError::Network(format!("Invalid bootstrap addr '{}': {}", peer_addr_str, e)))?;

        // Extract peer ID from the multiaddr if present (last /p2p/ component)
        if let Some(libp2p::multiaddr::Protocol::P2p(peer_id)) = addr.iter().last() {
            let peer_addr = addr
                .iter()
                .filter(|p| !matches!(p, libp2p::multiaddr::Protocol::P2p(_)))
                .collect::<Multiaddr>();

            swarm_guard
                .behaviour_mut()
                .kademlia
                .add_address(&peer_id, peer_addr);

            info!("Added bootstrap peer: {}", peer_addr_str);
        }
    }

    // Trigger Kademlia bootstrap
    if !config.bootstrap_peers.is_empty() {
        let _ = swarm_guard.behaviour_mut().kademlia.bootstrap();
        info!("Kademlia bootstrap initiated");
    }

    if config.enable_mdns {
        info!("mDNS discovery is active (auto-started with behaviour)");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transport::{setup_transport, TransportConfig};

    #[tokio::test]
    async fn discovery_with_empty_bootstrap() {
        let config = TransportConfig {
            listen_addr: "/ip4/127.0.0.1/tcp/0".to_string(),
            enable_quic: false,
        };
        let keypair = libp2p::identity::Keypair::generate_ed25519();
        let swarm = setup_transport(&config, keypair).await.unwrap();

        let disc_config = DiscoveryConfig {
            enable_mdns: true,
            bootstrap_peers: vec![],
        };
        let result = start_discovery(&swarm, &disc_config).await;
        assert!(result.is_ok());
    }
}
