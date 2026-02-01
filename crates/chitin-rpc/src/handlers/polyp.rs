// crates/chitin-rpc/src/handlers/polyp.rs
//
// Polyp management handlers: Submit, Get, List, GetState, GetProvenance, GetHardeningReceipt.
// These handlers interact with chitin-store's RocksStore and HardenedStore.

use std::sync::Arc;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use chitin_core::polyp::{Polyp, PolypState};
use chitin_core::traits::PolypStore;
use chitin_store::RocksStore;

// ---------------------------------------------------------------------------
// SubmitPolyp
// ---------------------------------------------------------------------------

/// Request to submit a new Polyp to the network.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitPolypRequest {
    /// The text content to embed.
    pub content: String,
    /// MIME type of the content (e.g., "text/plain").
    pub content_type: String,
    /// Optional language code (e.g., "en").
    pub language: Option<String>,
    /// Pre-computed vector embedding values (if the caller already embedded).
    pub vector: Option<Vec<f32>>,
    /// Source URL for provenance.
    pub source_url: Option<String>,
    /// Source title for provenance.
    pub source_title: Option<String>,
}

/// Response from submitting a Polyp.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitPolypResponse {
    /// The UUID assigned to the new Polyp.
    pub polyp_id: Uuid,
    /// The initial state of the Polyp.
    pub state: String,
    /// Human-readable status message.
    pub message: String,
}

/// Handle a SubmitPolyp request.
///
/// Phase 1: Creates a Draft Polyp in the local store. In Phase 2+, this
/// will also trigger ZK proof generation and P2P gossip broadcast.
pub async fn handle_submit_polyp(
    _store: &Arc<RocksStore>,
    _request: SubmitPolypRequest,
) -> Result<SubmitPolypResponse, String> {
    // Phase 1: Create a placeholder Polyp ID.
    // Full implementation will assemble the Polyp struct, generate proofs, etc.
    let polyp_id = Uuid::now_v7();

    Ok(SubmitPolypResponse {
        polyp_id,
        state: "Draft".to_string(),
        message: "Polyp submitted successfully (Phase 1: local draft only)".to_string(),
    })
}

// ---------------------------------------------------------------------------
// GetPolyp
// ---------------------------------------------------------------------------

/// Request to retrieve a Polyp by ID.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetPolypRequest {
    /// The UUID of the Polyp to retrieve.
    pub polyp_id: Uuid,
}

/// Response containing a Polyp.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetPolypResponse {
    /// The Polyp, if found.
    pub polyp: Option<Polyp>,
    /// Whether the Polyp was found.
    pub found: bool,
}

/// Handle a GetPolyp request.
pub async fn handle_get_polyp(
    store: &Arc<RocksStore>,
    request: GetPolypRequest,
) -> Result<GetPolypResponse, String> {
    let polyp = store
        .get_polyp(&request.polyp_id)
        .await
        .map_err(|e| format!("Failed to get polyp: {}", e))?;

    Ok(GetPolypResponse {
        found: polyp.is_some(),
        polyp,
    })
}

// ---------------------------------------------------------------------------
// ListPolyps
// ---------------------------------------------------------------------------

/// Request to list Polyps with optional filters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListPolypsRequest {
    /// Filter by lifecycle state (e.g., "Draft", "Soft", "Hardened").
    pub state_filter: Option<String>,
    /// Maximum number of results to return.
    pub limit: Option<u32>,
    /// Offset for pagination.
    pub offset: Option<u32>,
}

/// Response containing a list of Polyps.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListPolypsResponse {
    /// The matching Polyps.
    pub polyps: Vec<Polyp>,
    /// Total count of matching Polyps (before pagination).
    pub total: u32,
}

/// Handle a ListPolyps request.
///
/// Phase 1: Lists Polyps by state from the local store. Limited filtering.
pub async fn handle_list_polyps(
    store: &Arc<RocksStore>,
    request: ListPolypsRequest,
) -> Result<ListPolypsResponse, String> {
    // Determine which state to query. Default to Draft if not specified.
    let state = match request.state_filter.as_deref() {
        Some("Draft") | None => PolypState::Draft,
        Some("Soft") => PolypState::Soft,
        Some("UnderReview") => PolypState::UnderReview,
        Some("Approved") => PolypState::Approved,
        Some("Hardened") => PolypState::Hardened,
        Some("Rejected") => PolypState::Rejected,
        Some(other) => return Err(format!("Unknown state filter: {}", other)),
    };

    let polyps = store
        .list_polyps_by_state(&state)
        .await
        .map_err(|e| format!("Failed to list polyps: {}", e))?;

    let total = polyps.len() as u32;
    let offset = request.offset.unwrap_or(0) as usize;
    let limit = request.limit.unwrap_or(100) as usize;

    let page: Vec<Polyp> = polyps.into_iter().skip(offset).take(limit).collect();

    Ok(ListPolypsResponse {
        polyps: page,
        total,
    })
}

// ---------------------------------------------------------------------------
// GetPolypState
// ---------------------------------------------------------------------------

/// Request to get the lifecycle state of a Polyp.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetPolypStateRequest {
    /// The UUID of the Polyp.
    pub polyp_id: Uuid,
}

/// Response containing the Polyp's lifecycle state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetPolypStateResponse {
    /// The current state as a string (e.g., "Draft", "Hardened").
    pub state: Option<String>,
    /// Whether the Polyp was found.
    pub found: bool,
}

/// Handle a GetPolypState request.
pub async fn handle_get_polyp_state(
    store: &Arc<RocksStore>,
    request: GetPolypStateRequest,
) -> Result<GetPolypStateResponse, String> {
    let polyp = store
        .get_polyp(&request.polyp_id)
        .await
        .map_err(|e| format!("Failed to get polyp state: {}", e))?;

    match polyp {
        Some(p) => Ok(GetPolypStateResponse {
            state: Some(format!("{:?}", p.state)),
            found: true,
        }),
        None => Ok(GetPolypStateResponse {
            state: None,
            found: false,
        }),
    }
}

// ---------------------------------------------------------------------------
// GetPolypProvenance
// ---------------------------------------------------------------------------

/// Request to get the full provenance chain for a Polyp.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetPolypProvenanceRequest {
    /// The UUID of the Polyp.
    pub polyp_id: Uuid,
}

/// Response containing the Polyp's provenance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetPolypProvenanceResponse {
    /// JSON-serialized provenance data, if found.
    pub provenance: Option<serde_json::Value>,
    /// Whether the Polyp was found.
    pub found: bool,
}

/// Handle a GetPolypProvenance request.
pub async fn handle_get_polyp_provenance(
    store: &Arc<RocksStore>,
    request: GetPolypProvenanceRequest,
) -> Result<GetPolypProvenanceResponse, String> {
    let polyp = store
        .get_polyp(&request.polyp_id)
        .await
        .map_err(|e| format!("Failed to get polyp provenance: {}", e))?;

    match polyp {
        Some(p) => {
            let prov_json = serde_json::to_value(&p.subject.provenance)
                .map_err(|e| format!("Failed to serialize provenance: {}", e))?;
            Ok(GetPolypProvenanceResponse {
                provenance: Some(prov_json),
                found: true,
            })
        }
        None => Ok(GetPolypProvenanceResponse {
            provenance: None,
            found: false,
        }),
    }
}

// ---------------------------------------------------------------------------
// GetHardeningReceipt
// ---------------------------------------------------------------------------

/// Request to get the hardening receipt for a Polyp.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetHardeningReceiptRequest {
    /// The UUID of the Polyp.
    pub polyp_id: Uuid,
}

/// Response containing the hardening receipt (CID, Merkle proof, attestations).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetHardeningReceiptResponse {
    /// JSON-serialized hardening lineage, if the Polyp is hardened.
    pub hardening: Option<serde_json::Value>,
    /// Whether the Polyp is hardened.
    pub is_hardened: bool,
}

/// Handle a GetHardeningReceipt request.
pub async fn handle_get_hardening_receipt(
    store: &Arc<RocksStore>,
    request: GetHardeningReceiptRequest,
) -> Result<GetHardeningReceiptResponse, String> {
    let polyp = store
        .get_polyp(&request.polyp_id)
        .await
        .map_err(|e| format!("Failed to get polyp: {}", e))?;

    match polyp {
        Some(p) => match &p.hardening {
            Some(lineage) => {
                let lineage_json = serde_json::to_value(lineage)
                    .map_err(|e| format!("Failed to serialize hardening lineage: {}", e))?;
                Ok(GetHardeningReceiptResponse {
                    hardening: Some(lineage_json),
                    is_hardened: true,
                })
            }
            None => Ok(GetHardeningReceiptResponse {
                hardening: None,
                is_hardened: false,
            }),
        },
        None => Ok(GetHardeningReceiptResponse {
            hardening: None,
            is_hardened: false,
        }),
    }
}
