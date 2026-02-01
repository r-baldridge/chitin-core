// crates/chitin-core/src/identity.rs

use serde::{Deserialize, Serialize};

/// Identity of a node on the Chitin network.
///
/// Follows the coldkey/hotkey pattern:
/// - **Coldkey**: Long-term identity key, kept offline. Controls staking and rewards.
/// - **Hotkey**: Operational key, kept on the node. Signs messages and proofs.
///
/// The coldkey delegates authority to the hotkey. Compromising the hotkey
/// does not compromise staked funds (only the coldkey can unstake).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeIdentity {
    /// Coldkey public key (ed25519). Long-term identity.
    pub coldkey: [u8; 32],
    /// Hotkey public key (ed25519). Operational identity.
    pub hotkey: [u8; 32],
    /// DID derived from coldkey (e.g., "did:chitin:0xabc...").
    pub did: String,
    /// Node type.
    pub node_type: NodeType,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum NodeType {
    /// Produces Polyps.
    Coral,
    /// Validates and scores Polyps.
    Tide,
    /// Both producer and validator (allowed in Phase 1, restricted later).
    Hybrid,
}
