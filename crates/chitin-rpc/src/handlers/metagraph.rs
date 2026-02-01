// crates/chitin-rpc/src/handlers/metagraph.rs
//
// Metagraph query handlers: GetMetagraph, GetNodeMetrics, GetWeights, GetBonds.
// Phase 4: Wired to live MetagraphManager, WeightMatrix, and BondMatrix state.

use std::collections::HashMap;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use chitin_consensus::bonds::BondMatrix;
use chitin_consensus::epoch::EpochManager;
use chitin_consensus::metagraph::MetagraphManager;
use chitin_consensus::weights::WeightMatrix;

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
/// Phase 4: Reads from MetagraphManager if available.
pub async fn handle_get_metagraph(
    _request: GetMetagraphRequest,
    metagraph_manager: Option<&Arc<RwLock<MetagraphManager>>>,
) -> Result<GetMetagraphResponse, String> {
    if let Some(mm) = metagraph_manager {
        let mm = mm.read().await;
        if let Some(mg) = mm.current() {
            let nodes: Vec<MetagraphNodeEntry> = mg
                .nodes
                .iter()
                .map(|n| MetagraphNodeEntry {
                    uid: n.uid,
                    node_type: format!("{:?}", n.node_type),
                    stake: n.stake,
                    trust: n.trust,
                    consensus: n.consensus,
                    incentive: n.incentive,
                    emission: n.emission,
                    polyp_count: n.polyp_count,
                    active: n.active,
                })
                .collect();
            return Ok(GetMetagraphResponse {
                epoch: mg.epoch,
                nodes,
                total_stake: mg.total_stake,
                total_hardened_polyps: mg.total_hardened_polyps,
            });
        }
    }

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
/// Phase 4: Looks up node by UID in MetagraphManager.
pub async fn handle_get_node_metrics(
    request: GetNodeMetricsRequest,
    metagraph_manager: Option<&Arc<RwLock<MetagraphManager>>>,
) -> Result<GetNodeMetricsResponse, String> {
    if let Some(mm) = metagraph_manager {
        let mm = mm.read().await;
        if let Some(mg) = mm.current() {
            if let Some(node) = mg.nodes.iter().find(|n| n.uid == request.uid) {
                return Ok(GetNodeMetricsResponse {
                    found: true,
                    node: Some(MetagraphNodeEntry {
                        uid: node.uid,
                        node_type: format!("{:?}", node.node_type),
                        stake: node.stake,
                        trust: node.trust,
                        consensus: node.consensus,
                        incentive: node.incentive,
                        emission: node.emission,
                        polyp_count: node.polyp_count,
                        active: node.active,
                    }),
                });
            }
        }
    }

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
/// Phase 4: Reads from WeightMatrix and converts to sparse representation.
pub async fn handle_get_weights(
    request: GetWeightsRequest,
    weight_matrix: Option<&Arc<RwLock<WeightMatrix>>>,
    epoch_manager: Option<&Arc<RwLock<EpochManager>>>,
) -> Result<GetWeightsResponse, String> {
    let current_epoch = if let Some(em) = epoch_manager {
        em.read().await.current_epoch()
    } else {
        0
    };

    if let Some(wm) = weight_matrix {
        let wm = wm.read().await;
        let mut sparse: HashMap<u16, Vec<(u16, f64)>> = HashMap::new();

        for (v_idx, row) in wm.weights.iter().enumerate() {
            let v_uid = v_idx as u16;
            // Apply validator_uid filter if specified
            if let Some(filter_uid) = request.validator_uid {
                if v_uid != filter_uid {
                    continue;
                }
            }
            let entries: Vec<(u16, f64)> = row
                .iter()
                .enumerate()
                .filter(|(_, &w)| w > 0.0)
                .map(|(c_idx, &w)| (c_idx as u16, w))
                .collect();
            if !entries.is_empty() {
                sparse.insert(v_uid, entries);
            }
        }

        return Ok(GetWeightsResponse {
            epoch: current_epoch,
            weights: sparse,
        });
    }

    Ok(GetWeightsResponse {
        epoch: current_epoch,
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
/// Phase 4: Reads from BondMatrix and converts to sparse representation.
pub async fn handle_get_bonds(
    request: GetBondsRequest,
    bond_matrix: Option<&Arc<RwLock<BondMatrix>>>,
    epoch_manager: Option<&Arc<RwLock<EpochManager>>>,
) -> Result<GetBondsResponse, String> {
    let current_epoch = if let Some(em) = epoch_manager {
        em.read().await.current_epoch()
    } else {
        0
    };

    if let Some(bm) = bond_matrix {
        let bm = bm.read().await;
        let mut sparse: HashMap<u16, Vec<(u16, f64)>> = HashMap::new();

        for (v_idx, row) in bm.bonds.iter().enumerate() {
            let v_uid = v_idx as u16;
            if let Some(filter_uid) = request.validator_uid {
                if v_uid != filter_uid {
                    continue;
                }
            }
            let entries: Vec<(u16, f64)> = row
                .iter()
                .enumerate()
                .filter(|(_, &b)| b > 0.0)
                .map(|(c_idx, &b)| (c_idx as u16, b))
                .collect();
            if !entries.is_empty() {
                sparse.insert(v_uid, entries);
            }
        }

        return Ok(GetBondsResponse {
            epoch: current_epoch,
            bonds: sparse,
        });
    }

    Ok(GetBondsResponse {
        epoch: current_epoch,
        bonds: HashMap::new(),
    })
}
