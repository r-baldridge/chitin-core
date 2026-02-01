// crates/chitin-core/src/lib.rs
//
// chitin-core: Core types, traits, and crypto primitives for the Chitin Protocol.
//
// This is the leaf crate that all other crates in the workspace depend on.
// It defines the canonical data structures, error types, cryptographic helpers,
// and trait interfaces used throughout the Reefipedia system.

pub mod consensus;
pub mod crypto;
pub mod embedding;
pub mod error;
pub mod identity;
pub mod metagraph;
pub mod polyp;
pub mod provenance;
pub mod traits;

// Re-export key types for ergonomic access from downstream crates.
// Usage: `use chitin_core::Polyp;`

// Polyp types
pub use polyp::{Payload, Polyp, PolypState, PolypSubject, ProofPublicInputs, ZkProof};

// Embedding types
pub use embedding::{EmbeddingModelId, VectorEmbedding};

// Provenance types
pub use provenance::{PipelineStep, ProcessingPipeline, Provenance, SourceAttribution};

// Identity types
pub use identity::{NodeIdentity, NodeType};

// Consensus types
pub use consensus::{
    Attestation, ConsensusMetadata, HardeningLineage, PolypScores, ValidatorScore,
};

// Metagraph types
pub use metagraph::{NodeInfo, ReefMetagraph};

// Error type
pub use error::ChitinError;

// Traits
pub use traits::{PolypScorer, PolypStore, ProofVerifier, VectorIndex};
