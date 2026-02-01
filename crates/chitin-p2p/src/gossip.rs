// crates/chitin-p2p/src/gossip.rs
//
// GossipSub for Polyp broadcast across the Chitin Protocol mesh.

use chitin_core::{ChitinError, Polyp};
use libp2p::gossipsub::IdentTopic;
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::SwarmHandle;

/// The GossipSub topic for broadcasting Polyps.
pub const POLYP_TOPIC: &str = "chitin/polyps/v1";

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

/// Subscribe the local node to the Polyp GossipSub topic.
pub async fn subscribe_polyp_topic(swarm: &SwarmHandle) -> Result<(), ChitinError> {
    let topic = IdentTopic::new(POLYP_TOPIC);
    let mut swarm_guard = swarm.lock().await;
    swarm_guard
        .behaviour_mut()
        .gossipsub
        .subscribe(&topic)
        .map_err(|e| ChitinError::Network(format!("Failed to subscribe to topic: {}", e)))?;
    info!("Subscribed to GossipSub topic: {}", POLYP_TOPIC);
    Ok(())
}

/// Broadcast a Polyp to the gossip mesh.
///
/// Serializes the Polyp to JSON and publishes it to the
/// `chitin/polyps/v1` GossipSub topic.
pub async fn broadcast_polyp(swarm: &SwarmHandle, polyp: &Polyp) -> Result<(), ChitinError> {
    let json = serde_json::to_vec(polyp)
        .map_err(|e| ChitinError::Serialization(format!("Failed to serialize Polyp: {}", e)))?;

    let topic = IdentTopic::new(POLYP_TOPIC);
    let mut swarm_guard = swarm.lock().await;
    swarm_guard
        .behaviour_mut()
        .gossipsub
        .publish(topic, json)
        .map_err(|e| ChitinError::Network(format!("Failed to publish Polyp: {}", e)))?;

    info!("Broadcast Polyp via GossipSub");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn polyp_topic_constant() {
        assert_eq!(POLYP_TOPIC, "chitin/polyps/v1");
    }
}
