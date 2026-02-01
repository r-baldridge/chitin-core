// crates/chitin-core/src/metagraph.rs

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::identity::NodeType;

/// The global network state — all nodes, their stakes, trust, and performance.
///
/// Analogous to Bittensor's Metagraph. Updated every epoch.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReefMetagraph {
    /// Current epoch number.
    pub epoch: u64,
    /// Block height (if anchored to a chain).
    pub block: u64,
    /// All registered nodes.
    pub nodes: Vec<NodeInfo>,
    /// Total staked $CTN across all nodes.
    pub total_stake: u64,
    /// Total Polyps hardened to date.
    pub total_hardened_polyps: u64,
    /// Current emission rate (tokens per epoch).
    pub emission_rate: u64,
    /// Weight matrix W[validator_uid][coral_uid] = weight.
    /// Sparse representation: only non-zero entries.
    pub weights: HashMap<u16, Vec<(u16, f64)>>,
    /// Bond matrix B[validator_uid][coral_uid] = bond.
    pub bonds: HashMap<u16, Vec<(u16, f64)>>,
}

/// Information about a single node in the metagraph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeInfo {
    /// Network-assigned unique ID (0..n).
    pub uid: u16,
    /// Hotkey (operational identity).
    pub hotkey: [u8; 32],
    /// Coldkey (staking identity).
    pub coldkey: [u8; 32],
    /// Node type.
    pub node_type: NodeType,
    /// Total $CTN staked (own + delegated).
    pub stake: u64,
    /// Trust score (EMA of validation performance).
    pub trust: f64,
    /// Consensus score (agreement with other validators).
    pub consensus: f64,
    /// Incentive score (share of epoch rewards).
    pub incentive: f64,
    /// Emission received this epoch.
    pub emission: u64,
    /// Number of Polyps produced (Coral) or validated (Tide).
    pub polyp_count: u64,
    /// Last active epoch.
    pub last_active: u64,
    /// Network address (multiaddr).
    pub axon_addr: String,
    /// Whether currently registered and active.
    pub active: bool,
}
