// crates/chitin-p2p/src/axon.rs
//
// Axon: inbound request handler for the Chitin Protocol.

use chitin_core::ChitinError;
use tracing::info;

use crate::SwarmHandle;

/// An Axon listens for inbound requests from remote Dendrites.
///
/// In the Chitin Protocol, Coral Nodes expose Axons that respond
/// to ValidationQuery requests from Tide Nodes.
pub struct Axon {
    /// The multiaddr this Axon is listening on.
    pub addr: String,
    /// Whether the Axon is currently accepting requests.
    pub running: bool,
    /// Handle to the shared libp2p Swarm.
    swarm: Option<SwarmHandle>,
}

impl std::fmt::Debug for Axon {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Axon")
            .field("addr", &self.addr)
            .field("running", &self.running)
            .field("swarm", &self.swarm.as_ref().map(|_| "<SwarmHandle>"))
            .finish()
    }
}

impl Axon {
    /// Create a new Axon bound to the given address.
    pub fn new(addr: String) -> Self {
        Self {
            addr,
            running: false,
            swarm: None,
        }
    }

    /// Start the Axon with a SwarmHandle, beginning to accept inbound requests.
    ///
    /// Stores the SwarmHandle and sets the running flag. The actual request-response
    /// event handling is driven by the Swarm event loop (external to Axon).
    pub async fn start(&mut self, swarm: SwarmHandle) -> Result<(), ChitinError> {
        if self.running {
            return Ok(()); // Already running, idempotent
        }

        self.swarm = Some(swarm);
        self.running = true;
        info!("Axon started on {}", self.addr);
        Ok(())
    }

    /// Stop the Axon, ceasing to accept inbound requests.
    pub async fn stop(&mut self) -> Result<(), ChitinError> {
        if !self.running {
            return Ok(()); // Already stopped, idempotent
        }

        self.swarm = None;
        self.running = false;
        info!("Axon stopped on {}", self.addr);
        Ok(())
    }

    /// Check if the Axon is currently running.
    pub fn is_running(&self) -> bool {
        self.running
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn axon_new_is_not_running() {
        let axon = Axon::new("/ip4/0.0.0.0/tcp/9944".to_string());
        assert!(!axon.is_running());
    }

    #[tokio::test]
    async fn axon_stop_when_not_running_is_ok() {
        let mut axon = Axon::new("/ip4/0.0.0.0/tcp/9944".to_string());
        let result = axon.stop().await;
        assert!(result.is_ok());
    }
}
