// crates/chitin-p2p/src/behaviour.rs
//
// Composed NetworkBehaviour for the Chitin Protocol P2P layer.

use libp2p::identity::Keypair;
use libp2p::kad::store::MemoryStore;
use libp2p::request_response::ProtocolSupport;
use libp2p::StreamProtocol;
use libp2p::{gossipsub, identify, kad, mdns, request_response, swarm::NetworkBehaviour};
use std::time::Duration;

/// The composed network behaviour for the Chitin Protocol.
#[derive(NetworkBehaviour)]
pub struct ChitinBehaviour {
    /// GossipSub for Polyp broadcast across the mesh.
    pub gossipsub: gossipsub::Behaviour,
    /// Kademlia DHT for peer discovery and content routing.
    pub kademlia: kad::Behaviour<MemoryStore>,
    /// mDNS for local network peer discovery.
    pub mdns: mdns::tokio::Behaviour,
    /// Identify protocol for exchanging peer info.
    pub identify: identify::Behaviour,
    /// Request-response for Axon/Dendrite point-to-point communication.
    pub request_response: request_response::cbor::Behaviour<Vec<u8>, Vec<u8>>,
}

impl ChitinBehaviour {
    /// Create a new ChitinBehaviour with the given keypair.
    pub fn new(keypair: &Keypair) -> Result<Self, Box<dyn std::error::Error>> {
        let peer_id = keypair.public().to_peer_id();

        // GossipSub configuration
        let gossipsub_config = gossipsub::ConfigBuilder::default()
            .heartbeat_interval(Duration::from_secs(1))
            .validation_mode(gossipsub::ValidationMode::Strict)
            .build()
            .map_err(|e| format!("GossipSub config error: {}", e))?;
        let gossipsub = gossipsub::Behaviour::new(
            gossipsub::MessageAuthenticity::Signed(keypair.clone()),
            gossipsub_config,
        )
        .map_err(|e| format!("GossipSub behaviour error: {}", e))?;

        // Kademlia configuration
        let store = MemoryStore::new(peer_id);
        let kademlia = kad::Behaviour::new(peer_id, store);

        // mDNS for local network discovery
        let mdns = mdns::tokio::Behaviour::new(mdns::Config::default(), peer_id)?;

        // Identify protocol
        let identify = identify::Behaviour::new(identify::Config::new(
            "/chitin/id/1.0.0".to_string(),
            keypair.public(),
        ));

        // Request-response protocol for Axon/Dendrite
        let request_response = request_response::cbor::Behaviour::new(
            [(
                StreamProtocol::new("/chitin/req/1.0.0"),
                ProtocolSupport::Full,
            )],
            request_response::Config::default(),
        );

        Ok(Self {
            gossipsub,
            kademlia,
            mdns,
            identify,
            request_response,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_behaviour_succeeds() {
        let keypair = Keypair::generate_ed25519();
        let behaviour = ChitinBehaviour::new(&keypair);
        assert!(behaviour.is_ok());
    }
}
