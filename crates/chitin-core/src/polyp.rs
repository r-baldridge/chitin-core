// crates/chitin-core/src/polyp.rs

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::consensus::{ConsensusMetadata, HardeningLineage};
use crate::crypto;
use crate::embedding::{EmbeddingModelId, VectorEmbedding};
use crate::error::ChitinError;
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
    /// Ed25519 signature over signable bytes (Phase 2: cryptographic polyp signing).
    /// None for unsigned polyps (backward compatible).
    #[serde(default)]
    pub signature: Option<Vec<u8>>,
}

impl Polyp {
    /// Compute the signable bytes for this polyp.
    ///
    /// Returns SHA-256(id_bytes || content || vector_values_as_le_bytes || created_at_rfc3339).
    pub fn signable_bytes(&self) -> Vec<u8> {
        let mut hasher = Sha256::new();

        // id bytes
        hasher.update(self.id.as_bytes());

        // content
        hasher.update(self.subject.payload.content.as_bytes());

        // vector values as little-endian bytes
        for val in &self.subject.vector.values {
            hasher.update(val.to_le_bytes());
        }

        // created_at as RFC3339 string
        let created_str = self.created_at.to_rfc3339();
        hasher.update(created_str.as_bytes());

        hasher.finalize().to_vec()
    }

    /// Sign this polyp with the given ed25519 signing key.
    ///
    /// Computes signable_bytes, signs with ed25519, and stores the signature.
    pub fn sign(&mut self, signing_key: &[u8; 32]) -> Result<(), ChitinError> {
        let message = self.signable_bytes();
        let signature = crypto::sign_message(signing_key, &message)?;
        self.signature = Some(signature);
        Ok(())
    }

    /// Verify this polyp's signature against the given ed25519 public key.
    ///
    /// Returns `Ok(false)` if the polyp has no signature (unsigned).
    /// Returns `Ok(true)` if the signature is valid.
    /// Returns `Ok(false)` if the signature is invalid.
    pub fn verify_signature(&self, public_key: &[u8; 32]) -> Result<bool, ChitinError> {
        match &self.signature {
            None => Ok(false),
            Some(sig) => {
                let message = self.signable_bytes();
                crypto::verify_signature(public_key, &message, sig)
            }
        }
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::Keypair;
    use crate::identity::{NodeIdentity, NodeType};
    use crate::provenance::{
        PipelineStep, ProcessingPipeline, Provenance, SourceAttribution,
    };

    /// Helper to create a test polyp for signing tests.
    fn make_test_polyp() -> Polyp {
        let now = Utc::now();
        Polyp {
            id: Uuid::now_v7(),
            state: PolypState::Draft,
            subject: PolypSubject {
                payload: Payload {
                    content: "test content".to_string(),
                    content_type: "text/plain".to_string(),
                    language: Some("en".to_string()),
                },
                vector: VectorEmbedding {
                    values: vec![0.1, 0.2, 0.3],
                    model_id: EmbeddingModelId {
                        provider: "test".to_string(),
                        name: "test-model".to_string(),
                        weights_hash: [0u8; 32],
                        dimensions: 3,
                    },
                    quantization: "float32".to_string(),
                    normalization: "l2".to_string(),
                },
                provenance: Provenance {
                    creator: NodeIdentity {
                        coldkey: [0u8; 32],
                        hotkey: [0u8; 32],
                        did: "did:chitin:local".to_string(),
                        node_type: NodeType::Coral,
                    },
                    source: SourceAttribution {
                        source_cid: None,
                        source_url: None,
                        title: None,
                        license: None,
                        accessed_at: now,
                    },
                    pipeline: ProcessingPipeline {
                        steps: vec![PipelineStep {
                            name: "test".to_string(),
                            version: "0.1.0".to_string(),
                            params: serde_json::json!({}),
                        }],
                        duration_ms: 0,
                    },
                },
            },
            proof: ZkProof {
                proof_type: "placeholder".to_string(),
                proof_value: "0x00".to_string(),
                vk_hash: "0x00".to_string(),
                public_inputs: ProofPublicInputs {
                    text_hash: [0u8; 32],
                    vector_hash: [0u8; 32],
                    model_id: EmbeddingModelId {
                        provider: "test".to_string(),
                        name: "test-model".to_string(),
                        weights_hash: [0u8; 32],
                        dimensions: 3,
                    },
                },
                created_at: now,
            },
            consensus: None,
            hardening: None,
            created_at: now,
            updated_at: now,
            signature: None,
        }
    }

    #[test]
    fn test_sign_verify_roundtrip() {
        let keypair = Keypair::generate();
        let signing_key_bytes = keypair.signing_key.to_bytes();
        let pubkey_bytes = keypair.public_key_bytes();

        let mut polyp = make_test_polyp();
        polyp.sign(&signing_key_bytes).unwrap();

        assert!(polyp.signature.is_some());
        let valid = polyp.verify_signature(&pubkey_bytes).unwrap();
        assert!(valid, "Signature should verify after signing with matching keypair");
    }

    #[test]
    fn test_tampered_content_fails_verification() {
        let keypair = Keypair::generate();
        let signing_key_bytes = keypair.signing_key.to_bytes();
        let pubkey_bytes = keypair.public_key_bytes();

        let mut polyp = make_test_polyp();
        polyp.sign(&signing_key_bytes).unwrap();

        // Tamper with the content after signing.
        polyp.subject.payload.content = "tampered content".to_string();

        let valid = polyp.verify_signature(&pubkey_bytes).unwrap();
        assert!(!valid, "Signature should fail after content tampering");
    }

    #[test]
    fn test_unsigned_polyp_returns_false() {
        let keypair = Keypair::generate();
        let pubkey_bytes = keypair.public_key_bytes();

        let polyp = make_test_polyp();
        assert!(polyp.signature.is_none());

        let valid = polyp.verify_signature(&pubkey_bytes).unwrap();
        assert!(!valid, "Unsigned polyp should return Ok(false)");
    }

    #[test]
    fn test_serde_backward_compat_no_signature() {
        let polyp = make_test_polyp();
        let json = serde_json::to_string(&polyp).unwrap();

        // Remove the "signature" field from the JSON to simulate old format.
        let mut value: serde_json::Value = serde_json::from_str(&json).unwrap();
        if let Some(obj) = value.as_object_mut() {
            obj.remove("signature");
        }
        let old_json = serde_json::to_string(&value).unwrap();

        // Deserialize — signature should default to None.
        let deserialized: Polyp = serde_json::from_str(&old_json).unwrap();
        assert!(deserialized.signature.is_none());
    }
}
