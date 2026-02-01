// crates/chitin-core/src/embedding.rs

use serde::{Deserialize, Serialize};

/// Identifies a specific embedding model version.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct EmbeddingModelId {
    /// Model family (e.g., "openai", "bge", "nomic").
    pub provider: String,
    /// Model name (e.g., "text-embedding-3-small").
    pub name: String,
    /// Exact version hash (SHA-256 of model weights).
    pub weights_hash: [u8; 32],
    /// Output dimensionality.
    pub dimensions: u32,
}

/// Deterministic pseudo-embedding: hash text + dimension index to produce a
/// reproducible float vector, then L2-normalize. Identical text always yields
/// an identical vector (cosine similarity ~1.0). No ML model required.
pub fn hash_embedding(text: &str, dimensions: usize) -> Vec<f32> {
    use sha2::{Sha256, Digest};

    let mut raw = Vec::with_capacity(dimensions);
    for i in 0..dimensions {
        let mut hasher = Sha256::new();
        hasher.update(text.as_bytes());
        hasher.update(i.to_le_bytes());
        let hash = hasher.finalize();
        // Interpret first 4 bytes as u32, map to [-1, 1]
        let bits = u32::from_le_bytes([hash[0], hash[1], hash[2], hash[3]]);
        let val = (bits as f64 / u32::MAX as f64) * 2.0 - 1.0;
        raw.push(val as f32);
    }

    // L2-normalize
    let norm: f32 = raw.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm > 0.0 {
        for v in raw.iter_mut() {
            *v /= norm;
        }
    }

    raw
}

/// A vector embedding with full model provenance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorEmbedding {
    /// The raw float vector.
    pub values: Vec<f32>,
    /// Which model produced this vector.
    pub model_id: EmbeddingModelId,
    /// Quantization applied, if any (e.g., "float32", "int8", "binary").
    pub quantization: String,
    /// Normalization applied (e.g., "l2", "none").
    pub normalization: String,
}
