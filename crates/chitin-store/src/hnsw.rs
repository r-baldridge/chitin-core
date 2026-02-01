// crates/chitin-store/src/hnsw.rs
//
// In-memory vector index implementing the `VectorIndex` trait.
//
// Phase 1: Simple brute-force cosine similarity search over an in-memory
// HashMap of vectors. Sufficient for local development and small datasets.
//
// Phase 2: This will be replaced by a Qdrant client integration
// (`qdrant-client` crate) providing production-grade HNSW-based ANN search
// with persistence, filtering, and horizontal scaling.

use std::collections::HashMap;
use std::sync::RwLock;

use async_trait::async_trait;
use uuid::Uuid;

use chitin_core::error::ChitinError;
use chitin_core::traits::VectorIndex;

/// In-memory vector index using brute-force cosine similarity.
///
/// This is a Phase 1 placeholder. For production use, replace with
/// Qdrant integration (Phase 2) which provides HNSW-based ANN search,
/// on-disk persistence, payload filtering, and multi-node sharding.
#[derive(Debug)]
pub struct InMemoryVectorIndex {
    /// Map from Polyp UUID to its vector embedding.
    vectors: RwLock<HashMap<Uuid, Vec<f32>>>,
}

impl InMemoryVectorIndex {
    /// Create a new empty in-memory vector index.
    pub fn new() -> Self {
        Self {
            vectors: RwLock::new(HashMap::new()),
        }
    }

    /// Return the number of vectors currently stored.
    pub fn len(&self) -> usize {
        self.vectors
            .read()
            .expect("RwLock poisoned")
            .len()
    }

    /// Return whether the index is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl Default for InMemoryVectorIndex {
    fn default() -> Self {
        Self::new()
    }
}

/// Compute cosine similarity between two vectors.
///
/// Returns a value in [-1.0, 1.0]. Returns 0.0 if either vector has zero magnitude.
fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() {
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

#[async_trait]
impl VectorIndex for InMemoryVectorIndex {
    async fn upsert(&self, id: Uuid, vector: &[f32]) -> Result<(), ChitinError> {
        let mut store = self
            .vectors
            .write()
            .map_err(|e| ChitinError::Storage(format!("RwLock poisoned: {}", e)))?;
        store.insert(id, vector.to_vec());
        Ok(())
    }

    async fn search(&self, query: &[f32], top_k: usize) -> Result<Vec<(Uuid, f32)>, ChitinError> {
        let store = self
            .vectors
            .read()
            .map_err(|e| ChitinError::Storage(format!("RwLock poisoned: {}", e)))?;

        // Brute-force: compute cosine similarity against every stored vector.
        let mut scored: Vec<(Uuid, f32)> = store
            .iter()
            .map(|(id, vec)| (*id, cosine_similarity(query, vec)))
            .collect();

        // Sort by descending similarity.
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Return top-k results.
        scored.truncate(top_k);
        Ok(scored)
    }

    async fn delete(&self, id: &Uuid) -> Result<(), ChitinError> {
        let mut store = self
            .vectors
            .write()
            .map_err(|e| ChitinError::Storage(format!("RwLock poisoned: {}", e)))?;
        store.remove(id);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cosine_similarity_identical() {
        let v = vec![1.0, 2.0, 3.0];
        let sim = cosine_similarity(&v, &v);
        assert!((sim - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_cosine_similarity_orthogonal() {
        let a = vec![1.0, 0.0];
        let b = vec![0.0, 1.0];
        let sim = cosine_similarity(&a, &b);
        assert!(sim.abs() < 1e-6);
    }

    #[test]
    fn test_cosine_similarity_opposite() {
        let a = vec![1.0, 0.0];
        let b = vec![-1.0, 0.0];
        let sim = cosine_similarity(&a, &b);
        assert!((sim + 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_cosine_similarity_zero_vector() {
        let a = vec![1.0, 2.0];
        let b = vec![0.0, 0.0];
        let sim = cosine_similarity(&a, &b);
        assert_eq!(sim, 0.0);
    }

    #[test]
    fn test_cosine_similarity_different_lengths() {
        let a = vec![1.0, 2.0, 3.0];
        let b = vec![1.0, 2.0];
        let sim = cosine_similarity(&a, &b);
        assert_eq!(sim, 0.0);
    }
}
