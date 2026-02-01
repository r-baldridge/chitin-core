// crates/chitin-drift/src/detection.rs
//
// Semantic drift detection across embedding model versions.

use chitin_core::ChitinError;
use serde::{Deserialize, Serialize};

/// Metrics quantifying semantic drift between two embedding model versions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriftMetrics {
    /// Mean cosine similarity shift across a reference corpus.
    pub mean_cosine_shift: f64,
    /// Maximum cosine similarity shift (worst-case drift for any single Polyp).
    pub max_cosine_shift: f64,
    /// Number of Polyps whose cosine shift exceeds the drift threshold.
    pub affected_polyps: usize,
}

/// Detects semantic drift between embedding model versions.
///
/// Uses a reference corpus to measure how much vector space geometry
/// changes when switching models.
#[derive(Debug)]
pub struct DriftDetector {
    /// Reference texts used to measure drift.
    pub reference_corpus: Vec<String>,
    /// Threshold above which cosine shift is considered significant drift.
    pub drift_threshold: f64,
}

impl DriftDetector {
    /// Create a new DriftDetector with default threshold.
    pub fn new() -> Self {
        Self {
            reference_corpus: Vec::new(),
            drift_threshold: 0.01,
        }
    }

    /// Create a DriftDetector with a specific corpus and threshold.
    pub fn with_corpus(corpus: Vec<String>, threshold: f64) -> Self {
        Self {
            reference_corpus: corpus,
            drift_threshold: threshold,
        }
    }

    /// Detect drift between an old and new embedding model.
    ///
    /// For each reference text, embeds with both the old model (using old_model as salt)
    /// and new model (using new_model as salt), then computes cosine similarity
    /// between the pair. Drift = 1.0 - cosine_similarity.
    pub fn detect_drift(
        &self,
        old_model: &str,
        new_model: &str,
    ) -> Result<DriftMetrics, ChitinError> {
        if self.reference_corpus.is_empty() {
            return Ok(DriftMetrics {
                mean_cosine_shift: 0.0,
                max_cosine_shift: 0.0,
                affected_polyps: 0,
            });
        }

        let dimensions = 64; // Use small dims for drift detection
        let mut shifts = Vec::with_capacity(self.reference_corpus.len());

        for text in &self.reference_corpus {
            // Embed with old model (salt = old_model_id + text)
            let old_input = format!("{}:{}", old_model, text);
            let old_vec = chitin_core::hash_embedding(&old_input, dimensions);

            // Embed with new model (salt = new_model_id + text)
            let new_input = format!("{}:{}", new_model, text);
            let new_vec = chitin_core::hash_embedding(&new_input, dimensions);

            let cosine_sim = cosine_similarity(&old_vec, &new_vec);
            let shift = 1.0 - cosine_sim;
            shifts.push(shift);
        }

        let mean_cosine_shift = shifts.iter().sum::<f64>() / shifts.len() as f64;
        let max_cosine_shift = shifts
            .iter()
            .cloned()
            .fold(f64::NEG_INFINITY, f64::max);
        let affected_polyps = shifts
            .iter()
            .filter(|&&s| s > self.drift_threshold)
            .count();

        Ok(DriftMetrics {
            mean_cosine_shift,
            max_cosine_shift,
            affected_polyps,
        })
    }
}

impl Default for DriftDetector {
    fn default() -> Self {
        Self::new()
    }
}

/// Compute cosine similarity between two f32 vectors.
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f64 {
    assert_eq!(a.len(), b.len(), "Vectors must have same length");
    if a.is_empty() {
        return 0.0;
    }

    let dot: f64 = a.iter().zip(b.iter()).map(|(x, y)| *x as f64 * *y as f64).sum();
    let norm_a: f64 = a.iter().map(|x| (*x as f64) * (*x as f64)).sum::<f64>().sqrt();
    let norm_b: f64 = b.iter().map(|x| (*x as f64) * (*x as f64)).sum::<f64>().sqrt();

    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }

    dot / (norm_a * norm_b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_model_zero_drift() {
        let corpus = vec![
            "the quick brown fox".to_string(),
            "hello world".to_string(),
            "rust programming language".to_string(),
        ];
        let detector = DriftDetector::with_corpus(corpus, 0.01);
        let metrics = detector.detect_drift("model-a", "model-a").unwrap();
        assert!(
            metrics.mean_cosine_shift.abs() < 1e-10,
            "Same model should have zero drift, got {}",
            metrics.mean_cosine_shift
        );
        assert_eq!(metrics.affected_polyps, 0);
    }

    #[test]
    fn different_models_nonzero_drift() {
        let corpus = vec![
            "the quick brown fox".to_string(),
            "hello world".to_string(),
        ];
        let detector = DriftDetector::with_corpus(corpus, 0.01);
        let metrics = detector.detect_drift("model-a", "model-b").unwrap();
        assert!(
            metrics.mean_cosine_shift > 0.0,
            "Different models should have nonzero drift"
        );
        assert!(
            metrics.max_cosine_shift >= metrics.mean_cosine_shift,
            "Max should be >= mean"
        );
    }

    #[test]
    fn empty_corpus_returns_zero() {
        let detector = DriftDetector::new();
        let metrics = detector.detect_drift("a", "b").unwrap();
        assert_eq!(metrics.mean_cosine_shift, 0.0);
        assert_eq!(metrics.max_cosine_shift, 0.0);
        assert_eq!(metrics.affected_polyps, 0);
    }

    #[test]
    fn cosine_similarity_identical_vectors() {
        let v = vec![1.0f32, 2.0, 3.0];
        let sim = cosine_similarity(&v, &v);
        assert!((sim - 1.0).abs() < 1e-10);
    }

    #[test]
    fn cosine_similarity_orthogonal_vectors() {
        let a = vec![1.0f32, 0.0];
        let b = vec![0.0f32, 1.0];
        let sim = cosine_similarity(&a, &b);
        assert!(sim.abs() < 1e-10);
    }
}
