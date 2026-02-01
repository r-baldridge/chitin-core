// crates/chitin-p2p/src/transport.rs
//
// TCP/QUIC transport setup for the Chitin Protocol P2P layer.

use chitin_core::ChitinError;
use serde::{Deserialize, Serialize};

/// Configuration for the P2P transport layer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransportConfig {
    /// Multiaddr to listen on (e.g., "/ip4/0.0.0.0/tcp/9944").
    pub listen_addr: String,
    /// Whether to enable QUIC transport in addition to TCP.
    pub enable_quic: bool,
}

/// Set up the libp2p transport with the given configuration.
///
/// # Phase 2
/// This will initialize TCP and optionally QUIC transports with
/// Noise encryption and Yamux multiplexing via libp2p.
pub async fn setup_transport(_config: &TransportConfig) -> Result<(), ChitinError> {
    // Phase 2: Initialize libp2p Swarm with TCP/QUIC + Noise + Yamux
    todo!("Phase 2: setup_transport — initialize libp2p transport layer")
}
