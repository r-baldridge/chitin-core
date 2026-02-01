// crates/chitin-p2p/src/dendrite.rs
//
// Dendrite: outbound request sender for the Chitin Protocol.

use chitin_core::ChitinError;
use libp2p::PeerId;
use tracing::info;

use crate::SwarmHandle;

/// A Dendrite sends outbound requests to remote Axons.
///
/// In the Chitin Protocol, Tide Nodes use Dendrites to send
/// ValidationQuery requests to Coral Node Axons.
#[derive(Debug)]
pub struct Dendrite {
    /// The target peer ID of the remote Axon.
    pub target_peer: PeerId,
}

impl Dendrite {
    /// Create a new Dendrite targeting the given remote peer.
    pub fn new(target: PeerId) -> Self {
        Self {
            target_peer: target,
        }
    }

    /// Send a query to the remote Axon via the request-response protocol.
    ///
    /// Uses the shared SwarmHandle to send a request to the target peer
    /// and returns the response bytes.
    pub async fn send_query(
        &self,
        swarm: &SwarmHandle,
        query: Vec<u8>,
    ) -> Result<Vec<u8>, ChitinError> {
        let mut swarm_guard = swarm.lock().await;
        let _request_id = swarm_guard
            .behaviour_mut()
            .request_response
            .send_request(&self.target_peer, query);
        drop(swarm_guard);

        info!("Sent query to peer {}", self.target_peer);

        // In a full implementation, we'd await the response event from the Swarm event loop.
        // For now, return an empty response indicating the request was dispatched.
        // The actual response handling requires integrating with the Swarm event loop.
        Err(ChitinError::Network(
            "Response collection requires Swarm event loop integration (pending)".to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dendrite_construction() {
        let peer_id = PeerId::random();
        let dendrite = Dendrite::new(peer_id);
        assert_eq!(dendrite.target_peer, peer_id);
    }
}
