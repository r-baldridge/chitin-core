// crates/chitin-verify/src/prover.rs
//
// ProofGenerator: Generates ZK proofs attesting that Vector = Model(Text).
//
// Phase 1: Generates placeholder proofs by hashing the text and vector.
//          The proof_value field is filled with a placeholder hex string.
// Phase 3: Real SP1 proof generation will be gated behind a `sp1` feature flag.

use chrono::Utc;
use sha2::{Digest, Sha256};

use chitin_core::embedding::EmbeddingModelId;
use chitin_core::polyp::{ProofPublicInputs, ZkProof};

/// Generates ZK proofs for Polyp submissions.
///
/// In Phase 1, this produces placeholder proofs that contain valid hashes
/// but a stub proof value. Real ZK proving (SP1/Risc0) will be added in Phase 3.
pub struct ProofGenerator;

impl ProofGenerator {
    /// Create a new ProofGenerator.
    pub fn new() -> Self {
        Self
    }

    /// Generate a ZK proof attesting that `vector` was produced by running `model_id` on `text`.
    ///
    /// # Phase 1 Behavior
    /// - Computes SHA-256 hashes of the text and vector bytes.
    /// - Fills the ZkProof with placeholder proof_value and vk_hash.
    /// - The proof is structurally valid but does not contain a real ZK proof.
    ///
    /// # Phase 3 (TODO)
    /// - Run the embedding model inside an SP1 zkVM guest program.
    /// - Generate a real Groth16/STARK proof.
    /// - The proof_value will contain the actual proof bytes.
    pub fn generate_proof(
        &self,
        text: &str,
        vector: &[f32],
        model_id: &EmbeddingModelId,
    ) -> Result<ZkProof, chitin_core::error::ChitinError> {
        // Compute SHA-256 hash of the source text
        let text_hash = {
            let mut hasher = Sha256::new();
            hasher.update(text.as_bytes());
            let result = hasher.finalize();
            let mut hash = [0u8; 32];
            hash.copy_from_slice(&result);
            hash
        };

        // Compute SHA-256 hash of the vector bytes (IEEE 754 little-endian)
        let vector_hash = {
            let mut hasher = Sha256::new();
            for &val in vector {
                hasher.update(val.to_le_bytes());
            }
            let result = hasher.finalize();
            let mut hash = [0u8; 32];
            hash.copy_from_slice(&result);
            hash
        };

        // Phase 1: Generate a placeholder proof value by hashing (text_hash || vector_hash).
        // This is NOT a real ZK proof — it simply demonstrates the data flow.
        let placeholder_proof_value = {
            let mut hasher = Sha256::new();
            hasher.update(text_hash);
            hasher.update(vector_hash);
            hex::encode(hasher.finalize())
        };

        // Phase 1: Placeholder verification key hash.
        // In Phase 3, this will be the hash of the SP1 verification key for the embedding circuit.
        let placeholder_vk_hash = {
            let mut hasher = Sha256::new();
            hasher.update(b"chitin-placeholder-vk-v1");
            hex::encode(hasher.finalize())
        };

        let public_inputs = ProofPublicInputs {
            text_hash,
            vector_hash,
            model_id: model_id.clone(),
        };

        Ok(ZkProof {
            // Phase 1: placeholder proof type indicating this is not a real ZK proof
            proof_type: "PlaceholderV1".to_string(),
            proof_value: placeholder_proof_value,
            vk_hash: placeholder_vk_hash,
            public_inputs,
            created_at: Utc::now(),
        })
    }
}

impl Default for ProofGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_proof_produces_valid_structure() {
        let generator = ProofGenerator::new();
        let text = "The mitochondria is the powerhouse of the cell.";
        let vector = vec![0.1_f32, 0.2, 0.3, 0.4];
        let model_id = EmbeddingModelId {
            provider: "test".to_string(),
            name: "test-model".to_string(),
            weights_hash: [0u8; 32],
            dimensions: 4,
        };

        let proof = generator.generate_proof(text, &vector, &model_id).unwrap();

        assert_eq!(proof.proof_type, "PlaceholderV1");
        assert!(!proof.proof_value.is_empty());
        assert!(!proof.vk_hash.is_empty());
        assert_eq!(proof.public_inputs.model_id, model_id);
        // text_hash should be non-zero
        assert_ne!(proof.public_inputs.text_hash, [0u8; 32]);
        // vector_hash should be non-zero
        assert_ne!(proof.public_inputs.vector_hash, [0u8; 32]);
    }

    #[test]
    fn test_different_texts_produce_different_hashes() {
        let generator = ProofGenerator::new();
        let vector = vec![0.1_f32, 0.2, 0.3];
        let model_id = EmbeddingModelId {
            provider: "test".to_string(),
            name: "test-model".to_string(),
            weights_hash: [0u8; 32],
            dimensions: 3,
        };

        let proof1 = generator
            .generate_proof("text one", &vector, &model_id)
            .unwrap();
        let proof2 = generator
            .generate_proof("text two", &vector, &model_id)
            .unwrap();

        assert_ne!(
            proof1.public_inputs.text_hash,
            proof2.public_inputs.text_hash
        );
    }
}
