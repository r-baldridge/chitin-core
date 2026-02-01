// crates/chitin-p2p/src/dendrite.rs
//
// Dendrite: outbound request sender for the Chitin Protocol.
// A Dendrite sends queries to Axons on remote nodes (e.g., ValidationQuery).

use chitin_core::ChitinError;

/// A Dendrite sends outbound requests to remote Axons.
///
/// In the Chitin Protocol, Tide Nodes use Dendrites to send
/// ValidationQuery requests to Coral Node Axons.
#[derive(Debug)]
pub struct Dendrite {
    /// The target multiaddr of the remote Axon.
    pub target_addr: String,
}

impl Dendrite {
    /// Create a new Dendrite targeting the given remote Axon address.
    pub fn new(target: String) -> Self {
        Self {
            target_addr: target,
        }
    }

    /// Send a query to the remote Axon and await the response.
    ///
    /// # Phase 2
    /// This will open a libp2p stream to the target Axon, send the
    /// serialized query, and return the deserialized response bytes.
    pub async fn send_query(&self, _query: &[u8]) -> Result<Vec<u8>, ChitinError> {
        // Phase 2: Open libp2p stream, send query, receive response
        todo!("Phase 2: Dendrite::send_query — send request to remote Axon")
    }
}
