// crates/chitin-drift/src/versioning.rs
//
// Model version registry and namespace management for the Chitin Protocol.
//
// Tracks which embedding models are active, their version history,
// and activation epochs. Each model version defines a vector namespace.

use serde::{Deserialize, Serialize};

/// A specific version of an embedding model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelVersion {
    /// The model identifier (e.g., "bge/bge-small-en-v1.5").
    pub model_id: String,
    /// Sequential version number for this model.
    pub version: u32,
    /// The epoch at which this model version was activated on the network.
    pub activated_at_epoch: u64,
}

/// Registry tracking all model versions and their activation history.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionRegistry {
    /// All registered model versions, ordered by registration time.
    pub versions: Vec<ModelVersion>,
}

impl VersionRegistry {
    /// Create a new empty VersionRegistry.
    pub fn new() -> Self {
        Self {
            versions: Vec::new(),
        }
    }

    /// Register a new model version.
    pub fn register(&mut self, version: ModelVersion) {
        self.versions.push(version);
    }

    /// Get the current (latest) version of a model by its model_id.
    ///
    /// Returns the version with the highest version number for the given model_id.
    pub fn current_version(&self, model_id: &str) -> Option<&ModelVersion> {
        self.versions
            .iter()
            .filter(|v| v.model_id == model_id)
            .max_by_key(|v| v.version)
    }

    /// List all versions of a model by its model_id, ordered by version number.
    pub fn list_versions(&self, model_id: &str) -> Vec<&ModelVersion> {
        let mut versions: Vec<&ModelVersion> = self
            .versions
            .iter()
            .filter(|v| v.model_id == model_id)
            .collect();
        versions.sort_by_key(|v| v.version);
        versions
    }
}

impl Default for VersionRegistry {
    fn default() -> Self {
        Self::new()
    }
}
