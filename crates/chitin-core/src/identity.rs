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

impl NodeIdentity {
    /// Construct a `NodeIdentity` from raw public key bytes.
    ///
    /// Sets the hotkey and coldkey fields directly and derives the DID
    /// from the coldkey public key.
    pub fn from_keypairs(
        hotkey_pub: [u8; 32],
        coldkey_pub: [u8; 32],
        node_type: NodeType,
    ) -> Self {
        let did = Self::derive_did(&coldkey_pub);
        Self {
            coldkey: coldkey_pub,
            hotkey: hotkey_pub,
            did,
            node_type,
        }
    }

    /// Derive a DID (Decentralized Identifier) from a coldkey public key.
    ///
    /// Format: `did:chitin:<hex-encoded-coldkey-pubkey>`
    pub fn derive_did(coldkey_pub: &[u8; 32]) -> String {
        let hex: String = coldkey_pub.iter().map(|b| format!("{:02x}", b)).collect();
        format!("did:chitin:{}", hex)
    }

    /// Returns true if this identity is a placeholder (coldkey is all zeros).
    ///
    /// Placeholder identities are used when no real key material has been loaded.
    pub fn is_placeholder(&self) -> bool {
        self.coldkey == [0u8; 32]
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_keypairs_constructs_correctly() {
        let hotkey_pub = [1u8; 32];
        let coldkey_pub = [2u8; 32];
        let identity = NodeIdentity::from_keypairs(hotkey_pub, coldkey_pub, NodeType::Coral);

        assert_eq!(identity.hotkey, hotkey_pub);
        assert_eq!(identity.coldkey, coldkey_pub);
        assert_eq!(identity.node_type, NodeType::Coral);
        assert!(identity.did.starts_with("did:chitin:"));
        assert_eq!(identity.did, NodeIdentity::derive_did(&coldkey_pub));
    }

    #[test]
    fn test_derive_did_is_deterministic() {
        let coldkey_pub = [0xab; 32];
        let did1 = NodeIdentity::derive_did(&coldkey_pub);
        let did2 = NodeIdentity::derive_did(&coldkey_pub);
        assert_eq!(did1, did2);
        // Verify the hex encoding is correct
        assert!(did1.starts_with("did:chitin:"));
        assert!(did1.contains("abababab"));
    }

    #[test]
    fn test_is_placeholder() {
        let placeholder = NodeIdentity {
            coldkey: [0u8; 32],
            hotkey: [0u8; 32],
            did: "did:chitin:local".to_string(),
            node_type: NodeType::Coral,
        };
        assert!(placeholder.is_placeholder());

        let real = NodeIdentity::from_keypairs([1u8; 32], [2u8; 32], NodeType::Coral);
        assert!(!real.is_placeholder());
    }
}
