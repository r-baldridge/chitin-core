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
    /// Human-readable details.
    pub details: Option<String>,
}

/// Handle a GetHealth request.
///
/// Phase 1: Always returns healthy. Phase 2+ will check actual subsystem status.
pub async fn handle_get_health(_request: GetHealthRequest) -> Result<GetHealthResponse, String> {
    Ok(GetHealthResponse {
        status: "healthy".to_string(),
        storage_ok: true,
        p2p_ok: false, // Phase 2: P2P not yet active
        index_ok: true,
        details: Some("Phase 1: Local-only mode".to_string()),
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
/// Phase 1: Returns empty list since P2P is not active.
pub async fn handle_get_peers(_request: GetPeersRequest) -> Result<GetPeersResponse, String> {
    Ok(GetPeersResponse {
        peers: Vec::new(),
        count: 0,
    })
}
