// crates/chitin-daemon/src/sync_loop.rs
//
// Background pull-sync loop: periodically fetches polyp ID lists from
// peers and retrieves any missing polyps.

use std::collections::HashSet;
use std::sync::Arc;

use chitin_core::polyp::{Polyp, PolypState};
use chitin_core::traits::{PolypStore, VectorIndex};
use chitin_store::{InMemoryVectorIndex, RocksStore};
use uuid::Uuid;

use crate::peers::PeerRegistry;

/// Run the background sync loop.
///
/// Every `interval_secs`, iterates configured peers:
/// 1. Calls `peer/list_polyp_ids` to get remote UUID list
/// 2. Compares against local store
/// 3. Fetches missing polyps via `polyp/get`
/// 4. Saves + indexes locally
pub async fn run_sync_loop(
    registry: Arc<PeerRegistry>,
    store: Arc<RocksStore>,
    index: Arc<InMemoryVectorIndex>,
    interval_secs: u64,
) {
    let mut interval = tokio::time::interval(std::time::Duration::from_secs(interval_secs));

    loop {
        interval.tick().await;

        if let Err(e) = sync_once(&registry, &store, &index).await {
            tracing::warn!("Sync loop error: {}", e);
        }
    }
}

/// Perform a single sync round against all peers.
async fn sync_once(
    registry: &PeerRegistry,
    store: &Arc<RocksStore>,
    index: &Arc<InMemoryVectorIndex>,
) -> Result<(), String> {
    // Build set of local polyp IDs.
    let local_ids = get_local_polyp_ids(store).await?;

    let peers = registry.configured_peer_urls().to_vec();
    let client = registry.http_client();

    for peer_url in &peers {
        // Step 1: Get remote polyp ID list.
        let remote_ids = match fetch_remote_polyp_ids(client, peer_url).await {
            Ok(ids) => {
                registry.mark_peer(peer_url, true, None).await;
                ids
            }
            Err(e) => {
                tracing::debug!("Sync: could not reach peer {}: {}", peer_url, e);
                registry.mark_peer(peer_url, false, None).await;
                continue;
            }
        };

        // Step 2: Find missing IDs.
        let missing: Vec<Uuid> = remote_ids
            .into_iter()
            .filter(|id| !local_ids.contains(id))
            .collect();

        if missing.is_empty() {
            tracing::trace!("Sync: in sync with peer {}", peer_url);
            continue;
        }

        tracing::info!(
            "Sync: {} missing polyps from peer {}",
            missing.len(),
            peer_url
        );

        // Step 3: Fetch and store missing polyps.
        for polyp_id in missing {
            match fetch_remote_polyp(client, peer_url, polyp_id).await {
                Ok(Some(polyp)) => {
                    // Phase 2: Verify signature if present (soft enforcement).
                    if polyp.signature.is_some() {
                        let creator_hotkey = &polyp.subject.provenance.creator.hotkey;
                        match polyp.verify_signature(creator_hotkey) {
                            Ok(true) => {
                                tracing::debug!(
                                    "Sync: polyp {} signature verified",
                                    polyp_id
                                );
                            }
                            Ok(false) => {
                                tracing::warn!(
                                    "Sync: polyp {} has INVALID signature (soft enforcement, accepting anyway)",
                                    polyp_id
                                );
                            }
                            Err(e) => {
                                tracing::warn!(
                                    "Sync: polyp {} signature verification error: {} (accepting anyway)",
                                    polyp_id,
                                    e
                                );
                            }
                        }
                    }

                    let values = polyp.subject.vector.values.clone();

                    if let Err(e) = store.save_polyp(&polyp).await {
                        tracing::warn!("Sync: failed to save polyp {}: {}", polyp_id, e);
                        continue;
                    }

                    if let Err(e) = index.upsert(polyp_id, &values).await {
                        tracing::warn!("Sync: failed to index polyp {}: {}", polyp_id, e);
                    }

                    tracing::debug!("Sync: pulled polyp {} from {}", polyp_id, peer_url);
                }
                Ok(None) => {
                    tracing::debug!(
                        "Sync: polyp {} not found on peer {} (may have been deleted)",
                        polyp_id,
                        peer_url
                    );
                }
                Err(e) => {
                    tracing::warn!(
                        "Sync: failed to fetch polyp {} from {}: {}",
                        polyp_id,
                        peer_url,
                        e
                    );
                }
            }
        }
    }

    Ok(())
}

/// Get all local polyp IDs as a HashSet for fast lookup.
async fn get_local_polyp_ids(store: &Arc<RocksStore>) -> Result<HashSet<Uuid>, String> {
    let states = [
        PolypState::Draft,
        PolypState::Soft,
        PolypState::UnderReview,
        PolypState::Approved,
        PolypState::Hardened,
        PolypState::Rejected,
    ];

    let mut ids = HashSet::new();
    for state in &states {
        let polyps = store
            .list_polyps_by_state(state)
            .await
            .map_err(|e| format!("Failed to list local polyps: {}", e))?;
        for p in polyps {
            ids.insert(p.id);
        }
    }

    Ok(ids)
}

/// JSON-RPC response envelope for parsing peer responses.
#[derive(serde::Deserialize)]
struct JsonRpcResponse {
    success: bool,
    result: Option<serde_json::Value>,
    error: Option<String>,
}

/// Fetch the list of polyp IDs from a remote peer.
async fn fetch_remote_polyp_ids(
    client: &reqwest::Client,
    peer_url: &str,
) -> Result<Vec<Uuid>, String> {
    let request_body = serde_json::json!({
        "method": "peer/list_polyp_ids",
        "params": {}
    });

    let resp = client
        .post(peer_url)
        .json(&request_body)
        .send()
        .await
        .map_err(|e| format!("HTTP error: {}", e))?;

    let rpc_resp: JsonRpcResponse = resp
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    if !rpc_resp.success {
        return Err(rpc_resp.error.unwrap_or_else(|| "Unknown error".to_string()));
    }

    let result = rpc_resp.result.ok_or("No result in response")?;

    #[derive(serde::Deserialize)]
    struct ListResult {
        ids: Vec<Uuid>,
    }

    let list: ListResult =
        serde_json::from_value(result).map_err(|e| format!("Failed to parse ID list: {}", e))?;

    Ok(list.ids)
}

/// Fetch a single polyp from a remote peer by UUID.
async fn fetch_remote_polyp(
    client: &reqwest::Client,
    peer_url: &str,
    polyp_id: Uuid,
) -> Result<Option<Polyp>, String> {
    let request_body = serde_json::json!({
        "method": "polyp/get",
        "params": {
            "polyp_id": polyp_id
        }
    });

    let resp = client
        .post(peer_url)
        .json(&request_body)
        .send()
        .await
        .map_err(|e| format!("HTTP error: {}", e))?;

    let rpc_resp: JsonRpcResponse = resp
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    if !rpc_resp.success {
        return Err(rpc_resp.error.unwrap_or_else(|| "Unknown error".to_string()));
    }

    let result = rpc_resp.result.ok_or("No result in response")?;

    #[derive(serde::Deserialize)]
    struct GetResult {
        polyp: Option<Polyp>,
        #[allow(dead_code)]
        found: bool,
    }

    let get: GetResult =
        serde_json::from_value(result).map_err(|e| format!("Failed to parse polyp: {}", e))?;

    Ok(get.polyp)
}
