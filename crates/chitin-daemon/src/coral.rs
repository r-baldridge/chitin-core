// crates/chitin-daemon/src/coral.rs
//
// CoralNode: Polyp production pipeline for the Chitin Protocol.
//
// Coral Nodes ingest text, generate embeddings (placeholder in Phase 1),
// create ZK proofs (placeholder), assemble Polyps, and persist them to RocksDB.

use chrono::Utc;
use chitin_core::{
    EmbeddingModelId, NodeIdentity, NodeType, Payload, Polyp, PolypState, PolypSubject,
    PipelineStep, ProcessingPipeline, Provenance, ProofPublicInputs, SourceAttribution,
    VectorEmbedding, ZkProof,
};
use chitin_core::traits::PolypStore;
use chitin_store::RocksStore;
use std::sync::Arc;
use uuid::Uuid;

use crate::config::DaemonConfig;

/// A Coral Node that produces Polyps from ingested text.
pub struct CoralNode {
    #[allow(dead_code)]
    config: DaemonConfig,
    store: Arc<RocksStore>,
    /// Node identity for provenance (Phase 2).
    node_identity: Option<NodeIdentity>,
    /// Signing key for polyp signing (Phase 2).
    signing_key: Option<[u8; 32]>,
}

impl CoralNode {
    /// Create a new CoralNode, opening a RocksDB store at the configured data directory.
    pub fn new(config: &DaemonConfig) -> Result<Self, Box<dyn std::error::Error>> {
        let db_path = format!("{}/rocksdb", config.data_dir);
        let store = RocksStore::open(&db_path)?;
        Ok(Self {
            config: config.clone(),
            store: Arc::new(store),
            node_identity: None,
            signing_key: None,
        })
    }

    /// Set the node identity and optional signing key for provenance and polyp signing.
    pub fn with_identity(mut self, identity: NodeIdentity, signing_key: Option<[u8; 32]>) -> Self {
        self.node_identity = Some(identity);
        self.signing_key = signing_key;
        self
    }

    /// Get a reference to the underlying RocksStore.
    pub fn store(&self) -> Arc<RocksStore> {
        self.store.clone()
    }

    /// Start the Coral Node event loop.
    ///
    /// Phase 1: Logs startup and runs a sleep loop until shutdown signal.
    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        tracing::info!("Coral node started");
        tracing::info!("Listening for Polyp ingestion requests...");

        // Phase 1: simple event loop that sleeps and checks for shutdown.
        loop {
            tokio::select! {
                _ = tokio::signal::ctrl_c() => {
                    tracing::info!("Coral node received shutdown signal");
                    break;
                }
                _ = tokio::time::sleep(tokio::time::Duration::from_secs(10)) => {
                    tracing::debug!("Coral node heartbeat");
                }
            }
        }

        Ok(())
    }

    /// Ingest text and create a Draft Polyp.
    ///
    /// Assembles a Polyp with:
    /// - The provided text as payload
    /// - A placeholder embedding (zero vector)
    /// - A placeholder ZK proof
    /// - Placeholder provenance
    ///
    /// Saves the Polyp to the local RocksDB store and returns its UUID.
    pub async fn ingest_text(
        &self,
        text: &str,
        content_type: &str,
    ) -> Result<Uuid, Box<dyn std::error::Error>> {
        let now = Utc::now();
        let id = Uuid::now_v7();

        // Phase 1: placeholder embedding (384-dim zero vector, matching bge-small default).
        let embedding = VectorEmbedding {
            values: vec![0.0f32; 384],
            model_id: EmbeddingModelId {
                provider: "bge".to_string(),
                name: "bge-small-en-v1.5".to_string(),
                weights_hash: [0u8; 32],
                dimensions: 384,
            },
            quantization: "float32".to_string(),
            normalization: "l2".to_string(),
        };

        let payload = Payload {
            content: text.to_string(),
            content_type: content_type.to_string(),
            language: Some("en".to_string()),
        };

        // Use real identity for provenance if available, otherwise placeholder.
        let creator = self.node_identity.clone().unwrap_or(NodeIdentity {
            coldkey: [0u8; 32],
            hotkey: [0u8; 32],
            did: "did:chitin:local".to_string(),
            node_type: NodeType::Coral,
        });
        let provenance = Provenance {
            creator,
            source: SourceAttribution {
                source_cid: None,
                source_url: None,
                title: None,
                license: None,
                accessed_at: now,
            },
            pipeline: ProcessingPipeline {
                steps: vec![PipelineStep {
                    name: "ingest".to_string(),
                    version: "0.1.0".to_string(),
                    params: serde_json::json!({}),
                }],
                duration_ms: 0,
            },
        };

        let subject = PolypSubject {
            payload,
            vector: embedding,
            provenance,
        };

        // Phase 1: placeholder ZK proof.
        let proof = ZkProof {
            proof_type: "placeholder".to_string(),
            proof_value: "0x00".to_string(),
            vk_hash: "0x00".to_string(),
            public_inputs: ProofPublicInputs {
                text_hash: [0u8; 32],
                vector_hash: [0u8; 32],
                model_id: EmbeddingModelId {
                    provider: "bge".to_string(),
                    name: "bge-small-en-v1.5".to_string(),
                    weights_hash: [0u8; 32],
                    dimensions: 384,
                },
            },
            created_at: now,
        };

        let mut polyp = Polyp {
            id,
            state: PolypState::Draft,
            subject,
            proof,
            consensus: None,
            hardening: None,
            created_at: now,
            updated_at: now,
            signature: None,
        };

        // Sign the polyp if a signing key is available.
        if let Some(ref key) = self.signing_key {
            if let Err(e) = polyp.sign(key) {
                tracing::warn!("Failed to sign polyp {}: {}", id, e);
            } else {
                tracing::debug!("Signed polyp {} with hotkey", id);
            }
        }

        self.store.save_polyp(&polyp).await?;
        tracing::info!("Created Draft Polyp: {}", id);

        Ok(id)
    }
}
