// crates/chitin-p2p/src/gossip.rs
//
// GossipSub for Polyp broadcast across the Chitin Protocol mesh.

use chitin_core::{ChitinError, Polyp};
use serde::{Deserialize, Serialize};

/// Configuration for the GossipSub protocol.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GossipConfig {
    /// Target number of mesh peers.
    pub mesh_n: usize,
    /// Minimum mesh peers before requesting more.
    pub mesh_n_low: usize,
    /// Maximum mesh peers before pruning.
    pub mesh_n_high: usize,
}

/// Broadcast a Polyp to the gossip mesh.
///
/// # Phase 2
/// This will serialize the Polyp and publish it to the GossipSub topic
/// so all subscribed peers (Tide Nodes) receive it for validation.
pub async fn broadcast_polyp(_polyp: &Polyp) -> Result<(), ChitinError> {
    // Phase 2: Serialize polyp and publish to GossipSub topic
    todo!("Phase 2: broadcast_polyp — publish Polyp to GossipSub mesh")
}
