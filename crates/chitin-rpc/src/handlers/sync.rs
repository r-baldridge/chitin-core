// crates/chitin-rpc/src/handlers/sync.rs
//
// Sync status and trigger handlers: GetSyncStatus, TriggerSync.
// Phase 1: Stub implementations. Phase 2+ will wire into chitin-sync.

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// GetSyncStatus
// ---------------------------------------------------------------------------

/// Request for the current sync status.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetSyncStatusRequest {}

/// Response containing the sync status.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetSyncStatusResponse {
    /// Whether the node is synced with the network.
    pub is_synced: bool,
    /// Number of blocks behind the network tip (0 if synced).
    pub blocks_behind: u64,
    /// Number of peers currently syncing from.
    pub syncing_from_peers: u32,
    /// Percentage of sync completion (0.0 to 100.0).
    pub sync_progress_percent: f64,
    /// Estimated time to completion in seconds.
    pub estimated_time_seconds: Option<u64>,
}

/// Handle a GetSyncStatus request.
///
/// Phase 1: Reports as synced since there is no P2P network.
pub async fn handle_get_sync_status(
    _request: GetSyncStatusRequest,
) -> Result<GetSyncStatusResponse, String> {
    Ok(GetSyncStatusResponse {
        is_synced: true, // Phase 1: local-only mode is always synced
        blocks_behind: 0,
        syncing_from_peers: 0,
        sync_progress_percent: 100.0,
        estimated_time_seconds: None,
    })
}

// ---------------------------------------------------------------------------
// TriggerSync
// ---------------------------------------------------------------------------

/// Request to manually trigger a sync with peers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerSyncRequest {
    /// Optional: specific peer ID to sync from.
    pub peer_id: Option<String>,
    /// Whether to perform a full sync (vs. incremental).
    pub full_sync: Option<bool>,
}

/// Response from a sync trigger.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerSyncResponse {
    /// Whether the sync was triggered.
    pub triggered: bool,
    /// Human-readable message.
    pub message: String,
}

/// Handle a TriggerSync request.
///
/// Phase 1 stub: No P2P sync available.
pub async fn handle_trigger_sync(
    _request: TriggerSyncRequest,
) -> Result<TriggerSyncResponse, String> {
    // Phase 2: Trigger sync via chitin-sync
    Ok(TriggerSyncResponse {
        triggered: false,
        message: "Phase 1: P2P sync not yet available in local-only mode".to_string(),
    })
}
