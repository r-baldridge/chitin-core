// crates/chitin-rpc/src/handlers/metagraph.rs
//
// Metagraph query handlers: GetMetagraph, GetNodeMetrics, GetWeights, GetBonds.
// Phase 1: Stub implementations. Phase 2+ will read from the MetagraphManager.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// GetMetagraph
// ---------------------------------------------------------------------------

/// Request for the full metagraph snapshot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetMetagraphRequest {
    /// Epoch number to query. If omitted, returns the latest.
    pub epoch: Option<u64>,
}

/// A node entry in the metagraph response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetagraphNodeEntry {
    /// Network UID.
    pub uid: u16,
    /// Node type.
    pub node_type: String,
    /// Total stake in rao.
    pub stake: u64,
    /// Trust score.
    pub trust: f64,
    /// Consensus score.
    pub consensus: f64,
    /// Incentive score.
    pub incentive: f64,
    /// Emission received this epoch (in rao).
    pub emission: u64,
    /// Number of Polyps produced or validated.
    pub polyp_count: u64,
    /// Whether the node is active.
    pub active: bool,
}

/// Response containing the full metagraph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetMetagraphResponse {
    /// Current epoch number.
    pub epoch: u64,
    /// All nodes in the metagraph.
    pub nodes: Vec<MetagraphNodeEntry>,
    /// Total staked $CTN in rao.
    pub total_stake: u64,
    /// Total hardened Polyps.
    pub total_hardened_polyps: u64,
}

/// Handle a GetMetagraph request.
///
/// Phase 1 stub: Returns an empty metagraph.
pub async fn handle_get_metagraph(
    _request: GetMetagraphRequest,
) -> Result<GetMetagraphResponse, String> {
    // Phase 2: Read from chitin_consensus::MetagraphManager
    Ok(GetMetagraphResponse {
        epoch: 0,
        nodes: Vec::new(),
        total_stake: 0,
        total_hardened_polyps: 0,
    })
}

// ---------------------------------------------------------------------------
// GetNodeMetrics
// ---------------------------------------------------------------------------

/// Request for a specific node's metrics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetNodeMetricsRequest {
    /// Network UID of the node to query.
    pub uid: u16,
}

/// Response containing a node's metrics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetNodeMetricsResponse {
    /// Whether the node was found.
    pub found: bool,
    /// The node's metrics, if found.
    pub node: Option<MetagraphNodeEntry>,
}

/// Handle a GetNodeMetrics request.
///
/// Phase 1 stub: Returns not-found.
pub async fn handle_get_node_metrics(
    _request: GetNodeMetricsRequest,
) -> Result<GetNodeMetricsResponse, String> {
    // Phase 2: Look up node in the MetagraphManager
    Ok(GetNodeMetricsResponse {
        found: false,
        node: None,
    })
}

// ---------------------------------------------------------------------------
// GetWeights
// ---------------------------------------------------------------------------

/// Request for the weight matrix.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetWeightsRequest {
    /// Epoch to query. If omitted, returns the latest.
    pub epoch: Option<u64>,
    /// Optional: filter by validator UID.
    pub validator_uid: Option<u16>,
}

/// Response containing the weight matrix.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetWeightsResponse {
    /// The epoch these weights are from.
    pub epoch: u64,
    /// Sparse weight matrix: validator_uid -> [(coral_uid, weight)].
    pub weights: HashMap<u16, Vec<(u16, f64)>>,
}

/// Handle a GetWeights request.
///
/// Phase 1 stub: Returns empty weights.
pub async fn handle_get_weights(
    _request: GetWeightsRequest,
) -> Result<GetWeightsResponse, String> {
    // Phase 2: Read from the MetagraphManager
    Ok(GetWeightsResponse {
        epoch: 0,
        weights: HashMap::new(),
    })
}

// ---------------------------------------------------------------------------
// GetBonds
// ---------------------------------------------------------------------------

/// Request for the bond matrix.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetBondsRequest {
    /// Epoch to query. If omitted, returns the latest.
    pub epoch: Option<u64>,
    /// Optional: filter by validator UID.
    pub validator_uid: Option<u16>,
}

/// Response containing the bond matrix.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetBondsResponse {
    /// The epoch these bonds are from.
    pub epoch: u64,
    /// Sparse bond matrix: validator_uid -> [(coral_uid, bond)].
    pub bonds: HashMap<u16, Vec<(u16, f64)>>,
}

/// Handle a GetBonds request.
///
/// Phase 1 stub: Returns empty bonds.
pub async fn handle_get_bonds(_request: GetBondsRequest) -> Result<GetBondsResponse, String> {
    // Phase 2: Read from the MetagraphManager
    Ok(GetBondsResponse {
        epoch: 0,
        bonds: HashMap::new(),
    })
}
