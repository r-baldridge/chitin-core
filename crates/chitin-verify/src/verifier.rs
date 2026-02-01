// crates/chitin-verify/src/verifier.rs
//
// PlaceholderVerifier: Implements the ProofVerifier trait from chitin-core.
//
// Phase 1: Always returns Ok(true) — no real ZK verification is performed.
// Phase 3: Real SP1/Risc0 proof verification will replace the placeholder logic.

use sha2::{Digest, Sha256};

use chitin_core::error::ChitinError;
use chitin_core::polyp::ZkProof;
use chitin_core::traits::ProofVerifier;

/// A placeholder ZK proof verifier for Phase 1 development.
///
/// This verifier does NOT perform actual ZK proof verification.
/// It always returns `Ok(true)` to allow the rest of the system to develop
/// against a working proof pipeline.
///
/// In Phase 3, this will be replaced by `Sp1Verifier` and/or `Risc0Verifier`
/// that perform real cryptographic proof verification in constant time.
pub struct PlaceholderVerifier;

impl PlaceholderVerifier {
    /// Create a new PlaceholderVerifier.
    pub fn new() -> Self {
        Self
    }

    /// Verify that the text_hash in the proof's public inputs matches
    /// the SHA-256 hash of the given text.
    ///
    /// This check is independent of ZK proof verification — it validates
    /// that the public inputs are consistent with the claimed source text.
    pub fn verify_text_hash(proof: &ZkProof, text: &str) -> bool {
        let mut hasher = Sha256::new();
        hasher.update(text.as_bytes());
        let result = hasher.finalize();
        let mut expected = [0u8; 32];
        expected.copy_from_slice(&result);
        proof.public_inputs.text_hash == expected
    }

    /// Verify that the vector_hash in the proof's public inputs matches
    /// the SHA-256 hash of the given vector bytes (IEEE 754 little-endian).
    ///
    /// This check is independent of ZK proof verification — it validates
    /// that the public inputs are consistent with the claimed embedding vector.
    pub fn verify_vector_hash(proof: &ZkProof, vector: &[f32]) -> bool {
        let mut hasher = Sha256::new();
        for &val in vector {
            hasher.update(val.to_le_bytes());
        }
        let result = hasher.finalize();
        let mut expected = [0u8; 32];
        expected.copy_from_slice(&result);
        proof.public_inputs.vector_hash == expected
    }
}

impl Default for PlaceholderVerifier {
    fn default() -> Self {
        Self::new()
    }
}

impl ProofVerifier for PlaceholderVerifier {
    /// Verify a ZK proof.
    ///
    /// # Phase 1 Behavior
    /// Always returns `Ok(true)`. No actual ZK proof verification is performed.
    /// This allows the full Polyp lifecycle to be tested end-to-end without
    /// requiring a real zkVM setup.
    ///
    /// # Phase 3 (TODO)
    /// - Deserialize the proof bytes from `proof.proof_value`.
    /// - Load the verification key identified by `proof.vk_hash`.
    /// - Run the SP1/Risc0 verifier against the proof and public inputs.
    /// - Return `Ok(true)` only if cryptographic verification succeeds.
    fn verify_proof(&self, _proof: &ZkProof) -> Result<bool, ChitinError> {
        // Phase 1: placeholder — always accept.
        // TODO(Phase 3): Replace with real SP1/Risc0 verification:
        //   let vk = load_verification_key(&proof.vk_hash)?;
        //   let proof_bytes = hex::decode(&proof.proof_value)
        //       .map_err(|e| ChitinError::Verification(e.to_string()))?;
        //   sp1_sdk::verify(&vk, &proof_bytes, &proof.public_inputs)
        //       .map_err(|e| ChitinError::Verification(e.to_string()))
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prover::ProofGenerator;
    use chitin_core::embedding::EmbeddingModelId;

    fn test_model_id() -> EmbeddingModelId {
        EmbeddingModelId {
            provider: "test".to_string(),
            name: "test-model".to_string(),
            weights_hash: [0u8; 32],
            dimensions: 4,
        }
    }

    #[test]
    fn test_placeholder_verifier_always_returns_true() {
        let verifier = PlaceholderVerifier::new();
        let generator = ProofGenerator::new();
        let proof = generator
            .generate_proof("hello world", &[1.0, 2.0, 3.0, 4.0], &test_model_id())
            .unwrap();

        assert!(verifier.verify_proof(&proof).unwrap());
    }

    #[test]
    fn test_verify_text_hash_correct() {
        let generator = ProofGenerator::new();
        let text = "The quick brown fox";
        let proof = generator
            .generate_proof(text, &[1.0, 2.0], &test_model_id())
            .unwrap();

        assert!(PlaceholderVerifier::verify_text_hash(&proof, text));
    }

    #[test]
    fn test_verify_text_hash_incorrect() {
        let generator = ProofGenerator::new();
        let proof = generator
            .generate_proof("original text", &[1.0, 2.0], &test_model_id())
            .unwrap();

        assert!(!PlaceholderVerifier::verify_text_hash(&proof, "different text"));
    }

    #[test]
    fn test_verify_vector_hash_correct() {
        let generator = ProofGenerator::new();
        let vector = vec![1.0_f32, 2.0, 3.0, 4.0];
        let proof = generator
            .generate_proof("text", &vector, &test_model_id())
            .unwrap();

        assert!(PlaceholderVerifier::verify_vector_hash(&proof, &vector));
    }

    #[test]
    fn test_verify_vector_hash_incorrect() {
        let generator = ProofGenerator::new();
        let proof = generator
            .generate_proof("text", &[1.0, 2.0, 3.0], &test_model_id())
            .unwrap();

        assert!(!PlaceholderVerifier::verify_vector_hash(
            &proof,
            &[9.0, 8.0, 7.0]
        ));
    }
}
