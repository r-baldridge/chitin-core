// crates/chitin-consensus/src/scoring.rs
//
// Multi-dimensional Polyp scoring for the Chitin Protocol.
//
// Tide Nodes use this module to evaluate Polyps across five quality dimensions:
// ZK validity, semantic quality, novelty, source credibility, and embedding quality.

use chitin_core::{Polyp, PolypScores};

/// Score a Polyp across all five quality dimensions.
///
/// # Scoring Dimensions
/// 1. **ZK Validity** (0.0-1.0): 0.5 for placeholder proofs, 0.8 for non-placeholder.
/// 2. **Semantic Quality** (0.0-1.0): Content length heuristic.
/// 3. **Novelty** (0.0-1.0): Embedding variance proxy.
/// 4. **Source Credibility** (0.0-1.0): Provenance completeness check.
/// 5. **Embedding Quality** (0.0-1.0): Dimension match + L2 normalization check.
pub fn score_polyp_multi_dimensional(polyp: &Polyp) -> PolypScores {
    let zk_validity = score_zk_validity(polyp);
    let semantic_quality = score_semantic_quality(polyp);
    let novelty = score_novelty(polyp);
    let source_credibility = score_source_credibility(polyp);
    let embedding_quality = score_embedding_quality(polyp);

    PolypScores {
        zk_validity,
        semantic_quality,
        novelty,
        source_credibility,
        embedding_quality,
    }
}

/// ZK validity: 0.5 for placeholder proofs (all zeros or empty), 0.8 for non-placeholder.
fn score_zk_validity(polyp: &Polyp) -> f64 {
    let proof_bytes = polyp.proof.proof_value.as_bytes();
    let is_placeholder = proof_bytes.is_empty()
        || proof_bytes.iter().all(|&b| b == b'0');
    if is_placeholder {
        0.5
    } else {
        0.8
    }
}

/// Semantic quality: content length heuristic.
fn score_semantic_quality(polyp: &Polyp) -> f64 {
    let len = polyp.subject.payload.content.len();
    if len <= 10 {
        0.1
    } else if len <= 50 {
        0.3
    } else if len <= 200 {
        0.6
    } else if len <= 2000 {
        0.8
    } else {
        0.9
    }
}

/// Novelty: embedding variance proxy.
/// Zero vector -> 0.0; otherwise variance * 10 clamped to [0.0, 1.0].
fn score_novelty(polyp: &Polyp) -> f64 {
    let values = &polyp.subject.vector.values;
    if values.is_empty() || values.iter().all(|&v| v == 0.0) {
        return 0.0;
    }

    let n = values.len() as f64;
    let mean: f64 = values.iter().map(|&v| v as f64).sum::<f64>() / n;
    let variance: f64 = values.iter().map(|&v| {
        let diff = v as f64 - mean;
        diff * diff
    }).sum::<f64>() / n;

    (variance * 10.0).min(1.0).max(0.0)
}

/// Source credibility: provenance completeness check.
fn score_source_credibility(polyp: &Polyp) -> f64 {
    let prov = &polyp.subject.provenance;
    let mut score = 0.0;

    // source_url is Some and non-empty
    if let Some(ref url) = prov.source.source_url {
        if !url.is_empty() {
            score += 0.2;
        }
    }

    // title is Some and non-empty
    if let Some(ref title) = prov.source.title {
        if !title.is_empty() {
            score += 0.1;
        }
    }

    // Non-placeholder creator identity
    if prov.creator.coldkey != [0u8; 32] {
        score += 0.2;
    }

    // Pipeline steps: +0.1 per step, max +0.2
    let step_bonus = (prov.pipeline.steps.len() as f64 * 0.1).min(0.2);
    score += step_bonus;

    score.min(1.0)
}

/// Embedding quality: dimension match + L2 normalization + non-zero check.
fn score_embedding_quality(polyp: &Polyp) -> f64 {
    let vector = &polyp.subject.vector;
    let values = &vector.values;

    if values.is_empty() || values.iter().all(|&v| v == 0.0) {
        return 0.0;
    }

    let mut score: f64 = 0.0;

    // Check vector dimension matches model's expected dimensions
    if values.len() == vector.model_id.dimensions as usize {
        score += 0.5;
    }

    // Check if L2 norm is close to 1.0 (normalized)
    let l2_norm: f64 = values.iter().map(|&v| (v as f64) * (v as f64)).sum::<f64>().sqrt();
    if (l2_norm - 1.0).abs() < 0.1 {
        score += 0.3;
    }

    // Non-zero vector
    if values.iter().any(|&v| v != 0.0) {
        score += 0.2;
    }

    score.min(1.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chitin_core::{
        EmbeddingModelId, NodeIdentity, NodeType, Payload, PolypState, PolypSubject,
        ProcessingPipeline, Provenance, SourceAttribution, VectorEmbedding, ZkProof,
        ProofPublicInputs, PipelineStep,
    };
    use chrono::Utc;
    use uuid::Uuid;

    fn make_test_polyp(
        proof_value: &str,
        content: &str,
        vector_values: Vec<f32>,
        dimensions: u32,
    ) -> Polyp {
        make_test_polyp_full(proof_value, content, vector_values, dimensions, None, None, [0u8; 32], vec![])
    }

    fn make_test_polyp_full(
        proof_value: &str,
        content: &str,
        vector_values: Vec<f32>,
        dimensions: u32,
        source_url: Option<String>,
        title: Option<String>,
        coldkey: [u8; 32],
        pipeline_steps: Vec<PipelineStep>,
    ) -> Polyp {
        Polyp {
            id: Uuid::now_v7(),
            state: PolypState::Draft,
            subject: PolypSubject {
                payload: Payload {
                    content: content.to_string(),
                    content_type: "text/plain".to_string(),
                    language: Some("en".to_string()),
                },
                vector: VectorEmbedding {
                    values: vector_values,
                    model_id: EmbeddingModelId {
                        provider: "test".to_string(),
                        name: "test-model".to_string(),
                        weights_hash: [0u8; 32],
                        dimensions,
                    },
                    quantization: "float32".to_string(),
                    normalization: "l2".to_string(),
                },
                provenance: Provenance {
                    creator: NodeIdentity {
                        coldkey,
                        hotkey: [0u8; 32],
                        did: "did:chitin:test".to_string(),
                        node_type: NodeType::Coral,
                    },
                    source: SourceAttribution {
                        source_cid: None,
                        source_url,
                        title,
                        license: None,
                        accessed_at: Utc::now(),
                    },
                    pipeline: ProcessingPipeline {
                        steps: pipeline_steps,
                        duration_ms: 100,
                    },
                },
            },
            proof: ZkProof {
                proof_type: "SP1Groth16".to_string(),
                proof_value: proof_value.to_string(),
                vk_hash: "test_vk".to_string(),
                public_inputs: ProofPublicInputs {
                    text_hash: [0u8; 32],
                    vector_hash: [0u8; 32],
                    model_id: EmbeddingModelId {
                        provider: "test".to_string(),
                        name: "test-model".to_string(),
                        weights_hash: [0u8; 32],
                        dimensions,
                    },
                },
                created_at: Utc::now(),
            },
            consensus: None,
            hardening: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            signature: None,
        }
    }

    #[test]
    fn test_placeholder_proof_gets_zk_validity_half() {
        let polyp = make_test_polyp("0000000000", "test content", vec![0.1; 10], 10);
        let scores = score_polyp_multi_dimensional(&polyp);
        assert!((scores.zk_validity - 0.5).abs() < 1e-10);
    }

    #[test]
    fn test_short_content_low_semantic_quality() {
        let polyp = make_test_polyp("abc123", "hi", vec![0.1; 10], 10);
        let scores = score_polyp_multi_dimensional(&polyp);
        assert!((scores.semantic_quality - 0.1).abs() < 1e-10);
    }

    #[test]
    fn test_good_content_high_semantic_quality() {
        let content = "a".repeat(500);
        let polyp = make_test_polyp("abc123", &content, vec![0.1; 10], 10);
        let scores = score_polyp_multi_dimensional(&polyp);
        assert!((scores.semantic_quality - 0.8).abs() < 1e-10);
    }

    #[test]
    fn test_zero_vector_novelty_zero() {
        let polyp = make_test_polyp("abc123", "test content here", vec![0.0; 10], 10);
        let scores = score_polyp_multi_dimensional(&polyp);
        assert!((scores.novelty - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_normalized_embedding_high_quality() {
        // Create a normalized vector (L2 norm = 1.0)
        let dim = 4u32;
        let raw = vec![0.5f32, 0.5, 0.5, 0.5];
        let norm: f32 = raw.iter().map(|x| x * x).sum::<f32>().sqrt();
        let normalized: Vec<f32> = raw.iter().map(|x| x / norm).collect();

        let polyp = make_test_polyp("abc123", "test content", normalized, dim);
        let scores = score_polyp_multi_dimensional(&polyp);

        // dimension match (0.5) + L2 norm close to 1.0 (0.3) + non-zero (0.2) = 1.0
        assert!((scores.embedding_quality - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_full_integration_score_all_dimensions() {
        // Create a complete polyp with good provenance
        let dim = 8u32;
        let raw = vec![0.3f32, 0.4, 0.5, 0.2, 0.1, 0.6, 0.3, 0.2];
        let norm: f32 = raw.iter().map(|x| x * x).sum::<f32>().sqrt();
        let normalized: Vec<f32> = raw.iter().map(|x| x / norm).collect();

        let mut coldkey = [0u8; 32];
        coldkey[0] = 1; // non-placeholder

        let steps = vec![
            PipelineStep {
                name: "chunk".to_string(),
                version: "1.0".to_string(),
                params: serde_json::json!({}),
            },
            PipelineStep {
                name: "embed".to_string(),
                version: "1.0".to_string(),
                params: serde_json::json!({}),
            },
        ];

        let content = "This is a well-written piece of content that covers the topic in sufficient detail to be considered informative and high quality for the knowledge base.";
        let polyp = make_test_polyp_full(
            "abcdef1234567890",
            content,
            normalized,
            dim,
            Some("https://example.com/source".to_string()),
            Some("Test Article".to_string()),
            coldkey,
            steps,
        );

        let scores = score_polyp_multi_dimensional(&polyp);

        // Non-placeholder proof -> 0.8
        assert!((scores.zk_validity - 0.8).abs() < 1e-10);
        // Content length 158 chars -> 0.6 (len <= 200)
        assert!((scores.semantic_quality - 0.6).abs() < 1e-10);
        // Non-zero, varied vector -> novelty > 0.0
        assert!(scores.novelty > 0.0);
        // source_url(0.2) + title(0.1) + non-placeholder coldkey(0.2) + 2 steps(0.2) = 0.7
        assert!((scores.source_credibility - 0.7).abs() < 1e-10);
        // dimension match(0.5) + L2 norm ~1.0(0.3) + non-zero(0.2) = 1.0
        assert!((scores.embedding_quality - 1.0).abs() < 1e-10);
    }
}
