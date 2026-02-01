// crates/chitin-drift/src/molting.rs
//
// Molting orchestration: re-embed + re-prove for the Chitin Protocol.

use chitin_core::ChitinError;
use serde::{Deserialize, Serialize};

use crate::detection::DriftDetector;

/// Status of a molting operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MoltingStatus {
    /// Molting has been scheduled but not yet started.
    Pending,
    /// Molting is actively re-embedding Polyps.
    InProgress {
        /// Fraction of Polyps re-embedded (0.0 to 1.0).
        progress: f64,
    },
    /// All Polyps have been successfully re-embedded and re-proved.
    Completed,
    /// Molting failed with an error description.
    Failed(String),
}

/// Orchestrates the molting process for model migrations.
#[derive(Debug)]
pub struct MoltingOrchestrator {
    /// Drift detector used to check if molting is needed.
    drift_detector: DriftDetector,
}

impl MoltingOrchestrator {
    /// Create a new MoltingOrchestrator.
    pub fn new() -> Self {
        Self {
            drift_detector: DriftDetector::new(),
        }
    }

    /// Create a MoltingOrchestrator with a specific drift detector.
    pub fn with_detector(detector: DriftDetector) -> Self {
        Self {
            drift_detector: detector,
        }
    }

    /// Start a molting operation to migrate from an old model to a new model.
    ///
    /// If old_model == new_model, returns Completed immediately.
    /// If drift < 0.01, returns Completed (no significant drift).
    /// Otherwise, returns InProgress { progress: 0.0 } — actual batch
    /// re-embedding is deferred to a daemon background task.
    pub async fn start_molting(
        &self,
        old_model: &str,
        new_model: &str,
    ) -> Result<MoltingStatus, ChitinError> {
        // Same model: no molting needed
        if old_model == new_model {
            return Ok(MoltingStatus::Completed);
        }

        // Run drift detection
        let metrics = self.drift_detector.detect_drift(old_model, new_model)?;

        // If drift is below threshold, no molting needed
        if metrics.mean_cosine_shift < 0.01 {
            return Ok(MoltingStatus::Completed);
        }

        // Significant drift detected — initiate molting
        // Actual batch processing deferred to daemon task
        Ok(MoltingStatus::InProgress { progress: 0.0 })
    }
}

impl Default for MoltingOrchestrator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::detection::DriftDetector;

    #[tokio::test]
    async fn molting_same_model_completes() {
        let orch = MoltingOrchestrator::new();
        let status = orch.start_molting("model-a", "model-a").await.unwrap();
        assert!(matches!(status, MoltingStatus::Completed));
    }

    #[tokio::test]
    async fn molting_different_models_with_corpus_returns_in_progress() {
        let corpus = vec![
            "the quick brown fox".to_string(),
            "hello world".to_string(),
            "semantic drift detection".to_string(),
        ];
        let detector = DriftDetector::with_corpus(corpus, 0.01);
        let orch = MoltingOrchestrator::with_detector(detector);
        let status = orch.start_molting("model-a", "model-b").await.unwrap();
        assert!(
            matches!(status, MoltingStatus::InProgress { progress } if progress == 0.0),
            "Expected InProgress, got {:?}",
            status
        );
    }

    #[tokio::test]
    async fn molting_empty_corpus_different_models_completes() {
        // Empty corpus means zero drift, so molting completes
        let orch = MoltingOrchestrator::new();
        let status = orch.start_molting("model-a", "model-b").await.unwrap();
        assert!(matches!(status, MoltingStatus::Completed));
    }
}
