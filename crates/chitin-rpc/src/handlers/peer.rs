// crates/chitin-rpc/src/handlers/peer.rs
//
// Peer-to-peer relay handlers: Announce, ReceivePolyp, ListPolypIds.
// These endpoints enable HTTP-based polyp propagation between nodes.

use std::sync::Arc;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use chitin_core::traits::{PolypStore, VectorIndex};
use chitin_store::{InMemoryVectorIndex, RocksStore};

// ---------------------------------------------------------------------------
// peer/announce
// ---------------------------------------------------------------------------

/// Request for peer announcement (startup handshake).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnnounceRequest {
    /// The announcing node's DID or identifier.
    pub node_id: Option<String>,
    /// The announcing node's public URL.
    pub url: Option<String>,
}

/// Response to a peer announcement.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnnounceResponse {
    /// This node's DID or identifier.
    pub node_id: Option<String>,
    /// This node's public URL.
    pub url: Option<String>,
    /// Acknowledgement message.
    pub message: String,
}

/// Handle a peer/announce request.
///
/// Accepts a node_id and url from the announcing peer, returns this node's info.
/// In Phase 2, returns the real DID and self URL if available.
pub async fn handle_announce(
    request: AnnounceRequest,
) -> Result<AnnounceResponse, String> {
    tracing::info!(
        "Received peer announcement from node_id={:?} url={:?}",
        request.node_id,
        request.url
    );

    // Phase 2: DID and URL are injected via the service-level context.
    // The actual values are filled in by the dispatch layer that has access
    // to node_identity and self_url. This handler returns placeholders that
    // will be overridden by the dispatch layer if identity is available.
    Ok(AnnounceResponse {
        node_id: None, // Overridden by dispatch if identity is set
        url: None,     // Overridden by dispatch if self_url is set
        message: "Announcement received".to_string(),
    })
}

/// Handle a peer/announce request with node identity context.
///
/// This version receives the node's DID and self URL from the service layer
/// and includes them in the response.
pub async fn handle_announce_with_identity(
    request: AnnounceRequest,
    self_did: Option<String>,
    self_url: Option<String>,
) -> Result<AnnounceResponse, String> {
    tracing::info!(
        "Received peer announcement from node_id={:?} url={:?}",
        request.node_id,
        request.url
    );

    Ok(AnnounceResponse {
        node_id: self_did,
        url: self_url,
        message: "Announcement received".to_string(),
    })
}

// ---------------------------------------------------------------------------
// peer/receive_polyp
// ---------------------------------------------------------------------------

/// Request to receive a polyp from a peer (push propagation).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReceivePolypRequest {
    /// The full JSON-serialized Polyp.
    pub polyp: chitin_core::polyp::Polyp,
    /// The DID of the node that originally created this polyp.
    pub source_did: Option<String>,
}

/// Response to receiving a polyp.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReceivePolypResponse {
    /// Whether the polyp was accepted (new) or already existed.
    pub accepted: bool,
    /// Whether this was a duplicate.
    pub duplicate: bool,
    /// Status message.
    pub message: String,
}

/// Handle a peer/receive_polyp request.
///
/// Deduplicates by UUID — if the polyp already exists locally, it's a no-op.
/// If new, saves to store and indexes the vector.
pub async fn handle_receive_polyp(
    store: &Arc<RocksStore>,
    index: &Arc<InMemoryVectorIndex>,
    request: ReceivePolypRequest,
) -> Result<ReceivePolypResponse, String> {
    let polyp = request.polyp;
    let polyp_id = polyp.id;

    // Phase 2: Log signature verification status if polyp has a signature.
    if polyp.signature.is_some() {
        let creator_hotkey = &polyp.subject.provenance.creator.hotkey;
        match polyp.verify_signature(creator_hotkey) {
            Ok(true) => {
                tracing::info!("Received polyp {} with valid signature", polyp_id);
            }
            Ok(false) => {
                tracing::warn!(
                    "Received polyp {} with INVALID signature (soft enforcement)",
                    polyp_id
                );
            }
            Err(e) => {
                tracing::warn!(
                    "Received polyp {} signature verification error: {}",
                    polyp_id,
                    e
                );
            }
        }
    } else {
        tracing::debug!("Received unsigned polyp {} (backward compatible)", polyp_id);
    }

    // Dedup check: see if we already have this polyp.
    let existing = store
        .get_polyp(&polyp_id)
        .await
        .map_err(|e| format!("Failed to check polyp existence: {}", e))?;

    if existing.is_some() {
        tracing::debug!("Polyp {} already exists locally, skipping", polyp_id);
        return Ok(ReceivePolypResponse {
            accepted: false,
            duplicate: true,
            message: format!("Polyp {} already exists", polyp_id),
        });
    }

    // Extract vector values before saving (we need them for indexing).
    let values = polyp.subject.vector.values.clone();

    // Save to RocksDB.
    store
        .save_polyp(&polyp)
        .await
        .map_err(|e| format!("Failed to save received polyp: {}", e))?;

    // Index the vector.
    index
        .upsert(polyp_id, &values)
        .await
        .map_err(|e| format!("Failed to index received polyp: {}", e))?;

    tracing::info!(
        "Received and stored polyp {} from peer (source_did={:?})",
        polyp_id,
        request.source_did
    );

    Ok(ReceivePolypResponse {
        accepted: true,
        duplicate: false,
        message: format!("Polyp {} accepted and indexed", polyp_id),
    })
}

// ---------------------------------------------------------------------------
// peer/list_polyp_ids
// ---------------------------------------------------------------------------

/// Request to list all polyp UUIDs on this node.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListPolypIdsRequest {}

/// Response containing all local polyp UUIDs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListPolypIdsResponse {
    /// All polyp UUIDs stored on this node.
    pub ids: Vec<Uuid>,
    /// Total count.
    pub count: usize,
}

/// Handle a peer/list_polyp_ids request.
///
/// Returns all polyp UUIDs from the local store. Used by pull-sync to
/// find which polyps the remote has that we're missing.
pub async fn handle_list_polyp_ids(
    store: &Arc<RocksStore>,
    _request: ListPolypIdsRequest,
) -> Result<ListPolypIdsResponse, String> {
    // Collect IDs from all states.
    let states = [
        chitin_core::polyp::PolypState::Draft,
        chitin_core::polyp::PolypState::Soft,
        chitin_core::polyp::PolypState::UnderReview,
        chitin_core::polyp::PolypState::Approved,
        chitin_core::polyp::PolypState::Hardened,
        chitin_core::polyp::PolypState::Rejected,
    ];

    let mut all_ids = Vec::new();
    for state in &states {
        let polyps = store
            .list_polyps_by_state(state)
            .await
            .map_err(|e| format!("Failed to list polyps in state {:?}: {}", state, e))?;
        for p in polyps {
            all_ids.push(p.id);
        }
    }

    let count = all_ids.len();
    Ok(ListPolypIdsResponse { ids: all_ids, count })
}

// ---------------------------------------------------------------------------
// peer/discover
// ---------------------------------------------------------------------------

/// Request to discover known peers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoverPeersRequest {}

/// Information about a known peer in the discovery response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredPeer {
    /// The peer's URL.
    pub url: String,
    /// The peer's DID, if known.
    pub did: Option<String>,
    /// Whether the peer was reachable at last check.
    pub alive: bool,
}

/// Response containing known live peers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoverPeersResponse {
    /// Known peers with their URLs and DIDs.
    pub peers: Vec<DiscoveredPeer>,
    /// Total count.
    pub count: usize,
}

/// Handle a peer/discover request.
///
/// Returns the list of known live peer URLs and DIDs from the peer registry
/// information available to this node. The actual peer data is provided by
/// the service dispatch layer.
pub async fn handle_discover_peers(
    _request: DiscoverPeersRequest,
    peer_data: Vec<DiscoveredPeer>,
) -> Result<DiscoverPeersResponse, String> {
    let count = peer_data.len();
    Ok(DiscoverPeersResponse {
        peers: peer_data,
        count,
    })
}
