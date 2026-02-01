// crates/chitin-rpc/src/handlers/node.rs
//
// Node info and health handlers: GetNodeInfo, GetHealth, GetPeers.

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// GetNodeInfo
// ---------------------------------------------------------------------------

/// Request for node information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetNodeInfoRequest {}

/// Response containing node information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetNodeInfoResponse {
    /// Node type (e.g., "Coral", "Tide", "Hybrid").
    pub node_type: String,
    /// Software version.
    pub version: String,
    /// Uptime in seconds.
    pub uptime_seconds: u64,
    /// Node DID identifier.
    pub did: Option<String>,
    /// Capabilities list (e.g., ["polyp-submit", "query", "validate"]).
    pub capabilities: Vec<String>,
}

/// Handle a GetNodeInfo request.
///
/// Phase 1: Returns static placeholder info. Phase 2+ will return
/// actual node identity and capabilities from the daemon state.
pub async fn handle_get_node_info(
    _request: GetNodeInfoRequest,
) -> Result<GetNodeInfoResponse, String> {
    Ok(GetNodeInfoResponse {
        node_type: "Hybrid".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_seconds: 0, // Phase 2: track actual uptime
        did: None,
        capabilities: vec![
            "polyp-submit".to_string(),
            "query".to_string(),
            "local-store".to_string(),
        ],
    })
}

// ---------------------------------------------------------------------------
// GetHealth
// ---------------------------------------------------------------------------

/// Request for node health status.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetHealthRequest {}

/// Response containing node health status.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetHealthResponse {
    /// Overall health: "healthy", "degraded", or "unhealthy".
    pub status: String,
    /// RocksDB storage status.
    pub storage_ok: bool,
    /// P2P networking status.
    pub p2p_ok: bool,
    /// Vector index status.
    pub index_ok: bool,
    /// Number of configured peers (0 if peer networking is disabled).
    pub peer_count: usize,
    /// Human-readable details.
    pub details: Option<String>,
}

/// Handle a GetHealth request.
///
/// When peer_count > 0, reports p2p_ok as true.
pub async fn handle_get_health(
    _request: GetHealthRequest,
    peer_count: usize,
) -> Result<GetHealthResponse, String> {
    let p2p_ok = peer_count > 0;
    let details = if p2p_ok {
        format!("HTTP relay active: {} peers configured", peer_count)
    } else {
        "Local-only mode (no peers configured)".to_string()
    };

    Ok(GetHealthResponse {
        status: "healthy".to_string(),
        storage_ok: true,
        p2p_ok,
        index_ok: true,
        peer_count,
        details: Some(details),
    })
}

// ---------------------------------------------------------------------------
// GetPeers
// ---------------------------------------------------------------------------

/// Request for connected peers list.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetPeersRequest {}

/// Information about a connected peer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerInfo {
    /// Peer DID or identifier.
    pub peer_id: String,
    /// Multiaddr of the peer.
    pub address: String,
    /// Node type of the peer (if known).
    pub node_type: Option<String>,
    /// Connection latency in milliseconds.
    pub latency_ms: Option<u64>,
}

/// Response containing the list of connected peers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetPeersResponse {
    /// Connected peers.
    pub peers: Vec<PeerInfo>,
    /// Total number of connected peers.
    pub count: u32,
}

/// Handle a GetPeers request.
///
/// Returns the actual peer list from the peer registry (if configured).
pub async fn handle_get_peers(
    _request: GetPeersRequest,
    peer_data: Vec<PeerInfo>,
) -> Result<GetPeersResponse, String> {
    let count = peer_data.len() as u32;
    Ok(GetPeersResponse {
        peers: peer_data,
        count,
    })
}
