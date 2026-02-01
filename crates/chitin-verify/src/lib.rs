// crates/chitin-verify/src/lib.rs
//
// chitin-verify: ZK proof generation and verification for the Chitin Protocol.
//
// Phase 1: Placeholder implementations that generate and verify stub proofs.
// Phase 3: Real SP1/Risc0 proof generation behind feature flags.

pub mod models;
pub mod prover;
pub mod verifier;

// Re-export key types for ergonomic access from downstream crates.
pub use models::{ModelConfig, ModelRegistry};
pub use prover::ProofGenerator;
pub use verifier::PlaceholderVerifier;
