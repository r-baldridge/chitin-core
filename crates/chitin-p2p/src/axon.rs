// crates/chitin-p2p/src/axon.rs
//
// Axon: inbound request handler for the Chitin Protocol.
// An Axon serves queries from Dendrites on remote nodes (e.g., ValidationQuery).

use chitin_core::ChitinError;

/// An Axon listens for inbound requests from remote Dendrites.
///
/// In the Chitin Protocol, Coral Nodes expose Axons that respond
/// to ValidationQuery requests from Tide Nodes.
#[derive(Debug)]
pub struct Axon {
    /// The multiaddr this Axon is listening on.
    pub addr: String,
    /// Whether the Axon is currently accepting requests.
    pub running: bool,
}

impl Axon {
    /// Create a new Axon bound to the given address.
    pub fn new(addr: String) -> Self {
        Self {
            addr,
            running: false,
        }
    }

    /// Start the Axon, beginning to accept inbound requests.
    ///
    /// # Phase 2
    /// This will start a libp2p request-response protocol handler
    /// that serves ValidationQuery requests from Tide Nodes.
    pub async fn start(&mut self) -> Result<(), ChitinError> {
        // Phase 2: Start libp2p request-response handler
        todo!("Phase 2: Axon::start — begin accepting inbound requests")
    }

    /// Stop the Axon, ceasing to accept inbound requests.
    ///
    /// # Phase 2
    /// This will gracefully shut down the request-response handler
    /// and close all active connections.
    pub async fn stop(&mut self) -> Result<(), ChitinError> {
        // Phase 2: Gracefully shut down request-response handler
        todo!("Phase 2: Axon::stop — stop accepting inbound requests")
    }
}
