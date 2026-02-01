// crates/chitin-core/src/traits.rs

use async_trait::async_trait;
use uuid::Uuid;

use crate::consensus::PolypScores;
use crate::error::ChitinError;
use crate::polyp::{Polyp, PolypState, ZkProof};

/// Trait for persistent Polyp storage.
///
/// Implemented by chitin-store (RocksDB backend).
#[async_trait]
pub trait PolypStore: Send + Sync {
    /// Save a Polyp to the store. Overwrites if ID already exists.
    async fn save_polyp(&self, polyp: &Polyp) -> Result<(), ChitinError>;

    /// Retrieve a Polyp by its UUID.
    async fn get_polyp(&self, id: &Uuid) -> Result<Option<Polyp>, ChitinError>;

    /// List all Polyps in a given lifecycle state.
    async fn list_polyps_by_state(&self, state: &PolypState) -> Result<Vec<Polyp>, ChitinError>;

    /// Delete a Polyp by its UUID.
    async fn delete_polyp(&self, id: &Uuid) -> Result<(), ChitinError>;
}

/// Trait for ZK proof verification.
///
/// Implemented by chitin-verify.
pub trait ProofVerifier: Send + Sync {
    /// Verify a ZK proof. Returns `true` if the proof is valid.
    fn verify_proof(&self, proof: &ZkProof) -> Result<bool, ChitinError>;
}

/// Trait for multi-dimensional Polyp scoring.
///
/// Implemented by chitin-consensus scoring module.
pub trait PolypScorer: Send + Sync {
    /// Score a Polyp across all quality dimensions.
    fn score_polyp(&self, polyp: &Polyp) -> Result<PolypScores, ChitinError>;
}

/// Trait for vector similarity index operations.
///
/// Implemented by chitin-store (HNSW/Qdrant backend).
#[async_trait]
pub trait VectorIndex: Send + Sync {
    /// Insert or update a vector in the index.
    async fn upsert(&self, id: Uuid, vector: &[f32]) -> Result<(), ChitinError>;

    /// Search for the top-k nearest neighbors of a query vector.
    /// Returns a list of (UUID, similarity_score) pairs, sorted by descending similarity.
    async fn search(&self, query: &[f32], top_k: usize) -> Result<Vec<(Uuid, f32)>, ChitinError>;

    /// Delete a vector from the index by its UUID.
    async fn delete(&self, id: &Uuid) -> Result<(), ChitinError>;
}
