// crates/chitin-rpc/src/handlers/query.rs
//
// Query and retrieval handlers: SemanticSearch, HybridSearch, GetByCid, ExplainResult.
// These handlers interact with chitin-store's InMemoryVectorIndex and RocksStore.

use std::sync::Arc;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use chitin_core::hash_embedding;
use chitin_core::traits::{PolypStore, VectorIndex};
use chitin_store::{HardenedStore, InMemoryVectorIndex, RocksStore};

// ---------------------------------------------------------------------------
// SemanticSearch
// ---------------------------------------------------------------------------

/// Request for ANN semantic search.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticSearchRequest {
    /// Natural language query text.
    pub query_text: Option<String>,
    /// Pre-computed query vector (if the caller already embedded).
    pub query_vector: Option<Vec<f32>>,
    /// Which embedding model space to search in.
    pub model_id: Option<String>,
    /// Number of results to return (default 10).
    pub top_k: Option<u32>,
    /// Minimum trust score filter (default 0.0).
    pub min_trust: Option<f64>,
    /// Only return hardened Polyps (default true).
    pub hardened_only: Option<bool>,
    /// Topic filter (optional).
    pub reef_zone: Option<String>,
}

/// A single search result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    /// The Polyp UUID.
    pub polyp_id: Uuid,
    /// Cosine similarity score to the query.
    pub similarity: f32,
    /// The text content of the Polyp.
    pub content: Option<String>,
    /// The lifecycle state of the Polyp.
    pub state: String,
    /// CID if hardened.
    pub cid: Option<String>,
}

/// Response from a semantic search.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticSearchResponse {
    /// The search results, sorted by descending similarity.
    pub results: Vec<SearchResult>,
    /// Time taken for the search in milliseconds.
    pub search_time_ms: u64,
    /// Total results found before filtering.
    pub total_found: u32,
}

/// Handle a SemanticSearch request.
///
/// Searches the in-memory vector index for the nearest neighbors
/// of the query vector, then enriches results with Polyp data from the store.
pub async fn handle_semantic_search(
    store: &Arc<RocksStore>,
    index: &Arc<InMemoryVectorIndex>,
    request: SemanticSearchRequest,
) -> Result<SemanticSearchResponse, String> {
    let start = std::time::Instant::now();

    // Use provided vector or generate deterministic hash embedding from query text.
    let query_vector = match request.query_vector {
        Some(v) => v,
        None => match &request.query_text {
            Some(text) => hash_embedding(text, 384),
            None => {
                return Err("Either query_vector or query_text must be provided".to_string());
            }
        },
    };

    let top_k = request.top_k.unwrap_or(10) as usize;

    // Search the vector index.
    let raw_results = index
        .search(&query_vector, top_k)
        .await
        .map_err(|e| format!("Vector search failed: {}", e))?;

    let total_found = raw_results.len() as u32;

    // Enrich results with Polyp data from the store.
    let mut results = Vec::with_capacity(raw_results.len());
    for (polyp_id, similarity) in raw_results {
        let polyp = store
            .get_polyp(&polyp_id)
            .await
            .map_err(|e| format!("Failed to fetch polyp {}: {}", polyp_id, e))?;

        let (content, state, cid) = match polyp {
            Some(p) => {
                let content = Some(p.subject.payload.content.clone());
                let state = format!("{:?}", p.state);
                let cid = p.hardening.as_ref().map(|h| h.cid.clone());
                (content, state, cid)
            }
            None => (None, "Unknown".to_string(), None),
        };

        results.push(SearchResult {
            polyp_id,
            similarity,
            content,
            state,
            cid,
        });
    }

    let elapsed = start.elapsed().as_millis() as u64;

    Ok(SemanticSearchResponse {
        results,
        search_time_ms: elapsed,
        total_found,
    })
}

// ---------------------------------------------------------------------------
// HybridSearch
// ---------------------------------------------------------------------------

/// Request for hybrid (semantic + keyword) search.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HybridSearchRequest {
    /// Natural language query text for keyword matching.
    pub query_text: String,
    /// Pre-computed query vector for semantic matching.
    pub query_vector: Option<Vec<f32>>,
    /// Number of results to return (default 10).
    pub top_k: Option<u32>,
    /// Weight for semantic vs keyword: 0.0 = all keyword, 1.0 = all semantic.
    pub semantic_weight: Option<f64>,
}

/// Response from a hybrid search.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HybridSearchResponse {
    /// The search results, sorted by combined score.
    pub results: Vec<SearchResult>,
    /// Time taken for the search in milliseconds.
    pub search_time_ms: u64,
}

/// Handle a HybridSearch request.
///
/// Phase 1 stub: Falls back to semantic-only search if a vector is provided,
/// or returns an error explaining keyword search is not yet implemented.
pub async fn handle_hybrid_search(
    store: &Arc<RocksStore>,
    index: &Arc<InMemoryVectorIndex>,
    request: HybridSearchRequest,
) -> Result<HybridSearchResponse, String> {
    // Phase 1: If a vector is provided, delegate to semantic search.
    if let Some(vec) = request.query_vector {
        let semantic_request = SemanticSearchRequest {
            query_text: Some(request.query_text),
            query_vector: Some(vec),
            model_id: None,
            top_k: request.top_k,
            min_trust: None,
            hardened_only: None,
            reef_zone: None,
        };
        let resp = handle_semantic_search(store, index, semantic_request).await?;
        Ok(HybridSearchResponse {
            results: resp.results,
            search_time_ms: resp.search_time_ms,
        })
    } else {
        Err("Phase 1: Keyword-only search is not yet implemented. Provide a query_vector for semantic search.".to_string())
    }
}

// ---------------------------------------------------------------------------
// GetByCid
// ---------------------------------------------------------------------------

/// Request to retrieve a hardened Polyp by its IPFS CID.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetByCidRequest {
    /// The IPFS CID of the hardened Polyp.
    pub cid: String,
}

/// Response containing the hardened Polyp.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetByCidResponse {
    /// The Polyp data as JSON, if found.
    pub polyp: Option<serde_json::Value>,
    /// Whether the Polyp was found.
    pub found: bool,
}

/// Handle a GetByCid request.
///
/// Phase 4: Retrieves a hardened Polyp by CID from the HardenedStore.
pub async fn handle_get_by_cid(
    hardened_store: Option<&Arc<HardenedStore>>,
    request: GetByCidRequest,
) -> Result<GetByCidResponse, String> {
    match hardened_store {
        Some(hs) => {
            match hs.get_hardened(&request.cid).await {
                Ok(polyp) => {
                    let json = serde_json::to_value(&polyp)
                        .map_err(|e| format!("Failed to serialize polyp: {}", e))?;
                    Ok(GetByCidResponse {
                        polyp: Some(json),
                        found: true,
                    })
                }
                Err(_) => Ok(GetByCidResponse {
                    polyp: None,
                    found: false,
                }),
            }
        }
        None => {
            tracing::warn!(
                cid = %request.cid,
                "GetByCid: Hardened store not configured"
            );
            Ok(GetByCidResponse {
                polyp: None,
                found: false,
            })
        }
    }
}

// ---------------------------------------------------------------------------
// ExplainResult
// ---------------------------------------------------------------------------

/// Request to explain why a search result matched a query.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExplainResultRequest {
    /// The Polyp ID of the search result to explain.
    pub polyp_id: Uuid,
    /// The query vector used in the original search.
    pub query_vector: Vec<f32>,
}

/// Response explaining a search result match.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExplainResultResponse {
    /// Cosine similarity between the query and the Polyp's vector.
    pub cosine_similarity: f32,
    /// Dimensionality of the vectors.
    pub dimensions: u32,
    /// The Polyp's embedding model ID.
    pub model_id: Option<String>,
    /// Human-readable explanation.
    pub explanation: String,
}

/// Handle an ExplainResult request.
///
/// Computes and explains the similarity between a query vector
/// and a stored Polyp's vector.
pub async fn handle_explain_result(
    store: &Arc<RocksStore>,
    request: ExplainResultRequest,
) -> Result<ExplainResultResponse, String> {
    let polyp = store
        .get_polyp(&request.polyp_id)
        .await
        .map_err(|e| format!("Failed to get polyp: {}", e))?;

    match polyp {
        Some(p) => {
            let stored_vec = &p.subject.vector.values;
            let similarity = cosine_similarity_f32(&request.query_vector, stored_vec);
            let model_id = format!(
                "{}/{}",
                p.subject.vector.model_id.provider, p.subject.vector.model_id.name
            );

            Ok(ExplainResultResponse {
                cosine_similarity: similarity,
                dimensions: stored_vec.len() as u32,
                model_id: Some(model_id),
                explanation: format!(
                    "Cosine similarity: {:.4}. Vector dimensions: {}.",
                    similarity,
                    stored_vec.len()
                ),
            })
        }
        None => Err(format!("Polyp {} not found", request.polyp_id)),
    }
}

/// Compute cosine similarity between two f32 vectors.
fn cosine_similarity_f32(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }

    let mut dot = 0.0_f64;
    let mut norm_a = 0.0_f64;
    let mut norm_b = 0.0_f64;

    for (x, y) in a.iter().zip(b.iter()) {
        let x = *x as f64;
        let y = *y as f64;
        dot += x * y;
        norm_a += x * x;
        norm_b += y * y;
    }

    let denom = norm_a.sqrt() * norm_b.sqrt();
    if denom == 0.0 {
        return 0.0;
    }

    (dot / denom) as f32
}
