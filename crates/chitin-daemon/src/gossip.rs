// crates/chitin-daemon/src/gossip.rs
//
// Single-hop gossip broadcast: push a polyp to all configured peers.
// Fire-and-forget — failures are logged, never block the caller.

use std::sync::Arc;

use chitin_core::polyp::Polyp;

use crate::peers::PeerRegistry;

/// Broadcast a polyp to all configured peers via `peer/receive_polyp`.
///
/// For each peer, spawns an async task that POSTs the polyp.
/// Peers that are unreachable are logged and marked dead in the registry.
/// Peers do NOT re-broadcast (single-hop only).
pub fn broadcast_polyp(registry: Arc<PeerRegistry>, polyp: Polyp, source_did: Option<String>) {
    let peers = registry.configured_peer_urls().to_vec();

    if peers.is_empty() {
        return;
    }

    tracing::debug!(
        "Broadcasting polyp {} to {} peers",
        polyp.id,
        peers.len()
    );

    for peer_url in peers {
        let client = registry.http_client().clone();
        let reg = registry.clone();
        let polyp = polyp.clone();
        let source_did = source_did.clone();

        tokio::spawn(async move {
            let request_body = serde_json::json!({
                "method": "peer/receive_polyp",
                "params": {
                    "polyp": polyp,
                    "source_did": source_did,
                }
            });

            match client.post(&peer_url).json(&request_body).send().await {
                Ok(resp) => {
                    if resp.status().is_success() {
                        tracing::debug!("Pushed polyp {} to peer {}", polyp.id, peer_url);
                        reg.mark_peer(&peer_url, true, None).await;
                    } else {
                        tracing::warn!(
                            "Push polyp {} to peer {} returned status {}",
                            polyp.id,
                            peer_url,
                            resp.status()
                        );
                        reg.mark_peer(&peer_url, false, None).await;
                    }
                }
                Err(e) => {
                    tracing::warn!(
                        "Failed to push polyp {} to peer {}: {}",
                        polyp.id,
                        peer_url,
                        e
                    );
                    reg.mark_peer(&peer_url, false, None).await;
                }
            }
        });
    }
}
