// crates/chitin-daemon/src/peers.rs
//
// PeerRegistry: manages configured peer URLs and a shared HTTP client
// for inter-node communication in the HTTP relay network.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use serde::{Deserialize, Serialize};

/// Information about a peer node.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerState {
    /// The peer's public URL.
    pub url: String,
    /// The peer's self-reported node ID (DID), if known.
    pub node_id: Option<String>,
    /// Whether the last communication attempt succeeded.
    pub alive: bool,
}

/// Manages the set of known peers and a shared HTTP client.
#[derive(Debug, Clone)]
pub struct PeerRegistry {
    /// This node's public URL, used in announcements.
    pub self_url: Option<String>,
    /// This node's DID, included in announce messages.
    pub self_did: Option<String>,
    /// Configured peer URLs (from config).
    configured_peers: Vec<String>,
    /// Live peer state, updated on successful/failed communication.
    peer_state: Arc<RwLock<HashMap<String, PeerState>>>,
    /// Shared reqwest client for all outbound HTTP calls.
    client: reqwest::Client,
}

/// Request body for `peer/announce`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnnounceRequest {
    pub node_id: Option<String>,
    pub url: Option<String>,
}

/// Response body for `peer/announce`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnnounceResponse {
    pub node_id: Option<String>,
    pub url: Option<String>,
}

impl PeerRegistry {
    /// Create a new PeerRegistry from config values.
    pub fn new(self_url: Option<String>, configured_peers: Vec<String>) -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        let mut state_map = HashMap::new();
        for url in &configured_peers {
            state_map.insert(
                url.clone(),
                PeerState {
                    url: url.clone(),
                    node_id: None,
                    alive: false,
                },
            );
        }

        Self {
            self_url,
            self_did: None,
            configured_peers,
            peer_state: Arc::new(RwLock::new(state_map)),
            client,
        }
    }

    /// Return the shared reqwest::Client.
    pub fn http_client(&self) -> &reqwest::Client {
        &self.client
    }

    /// Return the list of configured peer URLs.
    pub fn configured_peer_urls(&self) -> &[String] {
        &self.configured_peers
    }

    /// Return the number of configured peers.
    #[allow(dead_code)]
    pub fn peer_count(&self) -> usize {
        self.configured_peers.len()
    }

    /// Return URLs of peers that last responded successfully.
    #[allow(dead_code)]
    pub async fn live_peer_urls(&self) -> Vec<String> {
        let state = self.peer_state.read().await;
        state
            .values()
            .filter(|p| p.alive)
            .map(|p| p.url.clone())
            .collect()
    }

    /// Return all peer states (for the peers RPC endpoint).
    #[allow(dead_code)]
    pub async fn all_peer_states(&self) -> Vec<PeerState> {
        let state = self.peer_state.read().await;
        state.values().cloned().collect()
    }

    /// Add a dynamically discovered peer if its URL is not already known.
    ///
    /// Returns `true` if the peer was newly added, `false` if it already existed.
    pub async fn add_discovered_peer(&self, url: String, did: Option<String>) -> bool {
        let mut state = self.peer_state.write().await;
        if state.contains_key(&url) {
            // Update DID if we got new info.
            if let (Some(peer), Some(new_did)) = (state.get_mut(&url), &did) {
                if peer.node_id.is_none() {
                    peer.node_id = Some(new_did.clone());
                }
            }
            return false;
        }

        tracing::info!("Discovered new peer: {} (did={:?})", url, did);
        state.insert(
            url.clone(),
            PeerState {
                url,
                node_id: did,
                alive: true,
            },
        );
        true
    }

    /// Mark a peer as alive or dead after a communication attempt.
    pub async fn mark_peer(&self, url: &str, alive: bool, node_id: Option<String>) {
        let mut state = self.peer_state.write().await;
        if let Some(peer) = state.get_mut(url) {
            peer.alive = alive;
            if let Some(id) = node_id {
                peer.node_id = Some(id);
            }
        }
    }

    /// Send `peer/announce` to all configured peers.
    /// Fire-and-forget: failures are logged, not propagated.
    pub async fn announce_to_all(&self) {
        let request_body = serde_json::json!({
            "method": "peer/announce",
            "params": {
                "node_id": self.self_did,
                "url": self.self_url,
            }
        });

        for peer_url in &self.configured_peers {
            let client = self.client.clone();
            let url = peer_url.clone();
            let body = request_body.clone();
            let registry = self.clone();

            tokio::spawn(async move {
                match client.post(&url).json(&body).send().await {
                    Ok(resp) => {
                        if resp.status().is_success() {
                            tracing::info!("Announced to peer {}", url);
                            registry.mark_peer(&url, true, None).await;
                        } else {
                            tracing::warn!(
                                "Announce to peer {} returned status {}",
                                url,
                                resp.status()
                            );
                            registry.mark_peer(&url, false, None).await;
                        }
                    }
                    Err(e) => {
                        tracing::warn!("Failed to announce to peer {}: {}", url, e);
                        registry.mark_peer(&url, false, None).await;
                    }
                }
            });
        }
    }
}
