// crates/chitin-verify/src/models.rs
//
// ModelRegistry: Registry of supported embedding models and their configurations.
//
// Each model has properties like dimensions, quantization, normalization,
// zkVM compatibility, and status. The registry is the source of truth for
// which models the network accepts for Polyp creation.

use serde::{Deserialize, Serialize};

/// Configuration for a single embedding model supported by the Chitin Protocol.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    /// Unique model identifier (e.g., "openai/text-embedding-3-small").
    pub id: String,
    /// Model provider (e.g., "openai", "bge", "nomic").
    pub provider: String,
    /// Model name (e.g., "text-embedding-3-small").
    pub name: String,
    /// Output vector dimensionality.
    pub dimensions: u32,
    /// Quantization format (e.g., "float32", "int8", "binary").
    pub quantization: String,
    /// Normalization applied (e.g., "l2", "none").
    pub normalization: String,
    /// SHA-256 hash of model weights (hex-encoded with "sha256:" prefix).
    pub weights_hash: String,
    /// Maximum input token length.
    pub max_tokens: u32,
    /// Whether this model can run inside a zkVM (SP1/Risc0).
    pub zkvm_compatible: bool,
    /// Target zkVM platform (e.g., "sp1", "risc0"). None if not zkVM-compatible.
    pub zkvm_target: Option<String>,
    /// Current status of the model in the registry.
    pub status: ModelStatus,
}

/// Status of a model in the registry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ModelStatus {
    /// Model is active and accepting new Polyps.
    Active,
    /// Model is deprecated — existing Polyps are valid but new ones are discouraged.
    Deprecated,
    /// Model has been retired — no new Polyps accepted, molting recommended.
    Retired,
}

/// Registry of supported embedding models.
///
/// The registry is the canonical source of truth for which models are available
/// in the network. Tide Nodes use it to verify that submitted Polyps reference
/// a valid model. Coral Nodes use it to select which model to embed with.
pub struct ModelRegistry {
    models: Vec<ModelConfig>,
}

impl ModelRegistry {
    /// Create a new empty ModelRegistry.
    pub fn new() -> Self {
        Self {
            models: Vec::new(),
        }
    }

    /// Load model configurations from a YAML file.
    ///
    /// # Phase 1
    /// Not yet implemented — serde_yaml is not in dependencies.
    /// Use `ModelRegistry::default()` for development.
    ///
    /// # Phase 2+
    /// Will parse `configs/model_configs.yaml` and populate the registry.
    pub fn load_from_yaml(_path: &str) -> Result<Self, chitin_core::error::ChitinError> {
        // TODO(Phase 2): Add serde_yaml dependency and implement YAML loading.
        //   let contents = std::fs::read_to_string(path)
        //       .map_err(|e| ChitinError::Storage(e.to_string()))?;
        //   let config: YamlConfig = serde_yaml::from_str(&contents)
        //       .map_err(|e| ChitinError::Serialization(e.to_string()))?;
        //   Ok(Self { models: config.models })
        todo!("YAML loading not yet implemented — serde_yaml not in dependencies. Use ModelRegistry::default() for Phase 1.")
    }

    /// Get the default model registry with the three models defined in
    /// ARCHITECTURE.md Section 8.1.
    ///
    /// These models represent the initial supported set for Phase 1 development:
    /// - OpenAI text-embedding-3-small (1536 dims)
    /// - BGE bge-small-en-v1.5 (384 dims) — default model
    /// - Nomic nomic-embed-text-v1.5 (768 dims)
    pub fn default_registry() -> Self {
        let models = vec![
            ModelConfig {
                id: "openai/text-embedding-3-small".to_string(),
                provider: "openai".to_string(),
                name: "text-embedding-3-small".to_string(),
                dimensions: 1536,
                quantization: "float32".to_string(),
                normalization: "l2".to_string(),
                weights_hash: "sha256:a1b2c3d4".to_string(),
                max_tokens: 8191,
                zkvm_compatible: true,
                zkvm_target: Some("sp1".to_string()),
                status: ModelStatus::Active,
            },
            ModelConfig {
                id: "bge/bge-small-en-v1.5".to_string(),
                provider: "bge".to_string(),
                name: "bge-small-en-v1.5".to_string(),
                dimensions: 384,
                quantization: "float32".to_string(),
                normalization: "l2".to_string(),
                weights_hash: "sha256:e5f6g7h8".to_string(),
                max_tokens: 512,
                zkvm_compatible: true,
                zkvm_target: Some("sp1".to_string()),
                status: ModelStatus::Active,
            },
            ModelConfig {
                id: "nomic/nomic-embed-text-v1.5".to_string(),
                provider: "nomic".to_string(),
                name: "nomic-embed-text-v1.5".to_string(),
                dimensions: 768,
                quantization: "float32".to_string(),
                normalization: "l2".to_string(),
                weights_hash: "sha256:i9j0k1l2".to_string(),
                max_tokens: 8192,
                zkvm_compatible: true,
                zkvm_target: Some("risc0".to_string()),
                status: ModelStatus::Active,
            },
        ];

        Self { models }
    }

    /// Look up a model by its identifier string (e.g., "bge/bge-small-en-v1.5").
    pub fn get_model(&self, id: &str) -> Option<&ModelConfig> {
        self.models.iter().find(|m| m.id == id)
    }

    /// List all models with `Active` status.
    pub fn list_active_models(&self) -> Vec<&ModelConfig> {
        self.models
            .iter()
            .filter(|m| m.status == ModelStatus::Active)
            .collect()
    }

    /// List all models in the registry regardless of status.
    pub fn list_all_models(&self) -> &[ModelConfig] {
        &self.models
    }

    /// Add a model to the registry.
    pub fn add_model(&mut self, config: ModelConfig) {
        self.models.push(config);
    }
}

impl Default for ModelRegistry {
    fn default() -> Self {
        Self::default_registry()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_registry_has_three_models() {
        let registry = ModelRegistry::default();
        assert_eq!(registry.list_all_models().len(), 3);
    }

    #[test]
    fn test_all_default_models_are_active() {
        let registry = ModelRegistry::default();
        assert_eq!(registry.list_active_models().len(), 3);
    }

    #[test]
    fn test_get_model_by_id() {
        let registry = ModelRegistry::default();

        let bge = registry.get_model("bge/bge-small-en-v1.5");
        assert!(bge.is_some());
        let bge = bge.unwrap();
        assert_eq!(bge.dimensions, 384);
        assert_eq!(bge.provider, "bge");

        let openai = registry.get_model("openai/text-embedding-3-small");
        assert!(openai.is_some());
        assert_eq!(openai.unwrap().dimensions, 1536);

        let nomic = registry.get_model("nomic/nomic-embed-text-v1.5");
        assert!(nomic.is_some());
        assert_eq!(nomic.unwrap().dimensions, 768);
    }

    #[test]
    fn test_get_model_not_found() {
        let registry = ModelRegistry::default();
        assert!(registry.get_model("nonexistent/model").is_none());
    }

    #[test]
    fn test_list_active_excludes_deprecated() {
        let mut registry = ModelRegistry::default();
        registry.add_model(ModelConfig {
            id: "test/deprecated-model".to_string(),
            provider: "test".to_string(),
            name: "deprecated-model".to_string(),
            dimensions: 128,
            quantization: "float32".to_string(),
            normalization: "l2".to_string(),
            weights_hash: "sha256:000".to_string(),
            max_tokens: 512,
            zkvm_compatible: false,
            zkvm_target: None,
            status: ModelStatus::Deprecated,
        });

        // 3 active from default + 0 from the deprecated addition
        assert_eq!(registry.list_active_models().len(), 3);
        // But total should be 4
        assert_eq!(registry.list_all_models().len(), 4);
    }
}
