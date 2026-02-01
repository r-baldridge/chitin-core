// crates/chitin-core/src/polyp.rs

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::consensus::{ConsensusMetadata, HardeningLineage};
use crate::embedding::{EmbeddingModelId, VectorEmbedding};
use crate::provenance::Provenance;

/// Lifecycle states of a Polyp — from initial creation through consensus to hardening.
///
///   Draft --> Soft --> UnderReview --> Approved --> Hardened
///                          |                           ^
///                          v                           |
///                       Rejected                    (immutable)
///                                                     |
///                                                   Molted (re-embedded with new model)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PolypState {
    /// Initial creation, not yet submitted to network.
    Draft,
    /// Submitted to network, ZK proof attached, awaiting validator pickup.
    Soft,
    /// Currently being evaluated by Tide Nodes in an active epoch.
    UnderReview,
    /// Passed Yuma-Semantic Consensus with sufficient validator agreement.
    Approved,
    /// Fully hardened: IPFS-pinned, CID anchored on-chain, attestations recorded.
    Hardened,
    /// Rejected by consensus — insufficient quality or failed ZK verification.
    Rejected,
    /// Superseded by a re-embedding under a newer model version (molting).
    Molted { successor_id: Uuid },
}

/// The atomic unit of knowledge in Reefipedia.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Polyp {
    /// Unique identifier (UUID v7 for time-ordering).
    pub id: Uuid,
    /// Current lifecycle state.
    pub state: PolypState,
    /// The knowledge content: text + embedding + provenance.
    pub subject: PolypSubject,
    /// ZK proof attesting Vector = Model(Text).
    pub proof: ZkProof,
    /// Consensus metadata (populated after validation).
    pub consensus: Option<ConsensusMetadata>,
    /// Hardening lineage (populated after hardening).
    pub hardening: Option<HardeningLineage>,
    /// Creation timestamp.
    pub created_at: DateTime<Utc>,
    /// Last state transition timestamp.
    pub updated_at: DateTime<Utc>,
}

/// The subject of a Polyp: payload (human-readable) + vector (machine-readable).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolypSubject {
    /// Human-readable knowledge content.
    pub payload: Payload,
    /// Machine-readable vector embedding.
    pub vector: VectorEmbedding,
    /// Full provenance chain.
    pub provenance: Provenance,
}

/// The human-readable knowledge content.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Payload {
    /// The raw text, code snippet, or structured data.
    pub content: String,
    /// MIME type of the content (e.g., "text/plain", "text/markdown", "application/json").
    pub content_type: String,
    /// Optional: language code (e.g., "en", "es").
    pub language: Option<String>,
}

/// ZK proof attesting to correct embedding generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZkProof {
    /// Proof system identifier: "SP1Groth16", "Risc0Stark", etc.
    pub proof_type: String,
    /// Hex-encoded proof bytes.
    pub proof_value: String,
    /// The verification key hash (identifies the circuit).
    pub vk_hash: String,
    /// Public inputs committed in the proof.
    pub public_inputs: ProofPublicInputs,
    /// Timestamp of proof generation.
    pub created_at: DateTime<Utc>,
}

/// Public inputs committed inside the ZK proof.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofPublicInputs {
    /// SHA-256 hash of the source text.
    pub text_hash: [u8; 32],
    /// SHA-256 hash of the resulting vector bytes.
    pub vector_hash: [u8; 32],
    /// Embedding model identifier.
    pub model_id: EmbeddingModelId,
}
