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
