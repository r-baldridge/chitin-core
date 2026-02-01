// crates/chitin-consensus/src/weights.rs
//
// Weight matrix management and normalization for the Chitin Protocol.
//
// The weight matrix W[validator][coral] stores each validator's score
// assignment for each Coral Node in the current epoch.

use serde::{Deserialize, Serialize};

/// A dense weight matrix where W[validator_idx][coral_idx] = weight.
///
/// Weights represent a Tide Node's assessment of a Coral Node's Polyp quality
/// in the current epoch. Weights are normalized per-validator to sum to 1.0.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeightMatrix {
    /// Dense weight matrix: weights[validator_idx][coral_idx].
    pub weights: Vec<Vec<f64>>,
}

impl WeightMatrix {
    /// Create a new zero-initialized weight matrix.
    ///
    /// # Arguments
    /// * `validators` - Number of validators (Tide Nodes).
    /// * `corals` - Number of Coral Nodes.
    pub fn new(validators: usize, corals: usize) -> Self {
        Self {
            weights: vec![vec![0.0; corals]; validators],
        }
    }

    /// Set the weight for validator `v` scoring coral `c`.
    pub fn set(&mut self, v: usize, c: usize, w: f64) {
        self.weights[v][c] = w;
    }

    /// Get the weight for validator `v` scoring coral `c`.
    pub fn get(&self, v: usize, c: usize) -> f64 {
        self.weights[v][c]
    }

    /// Normalize each validator's weight row to sum to 1.0.
    ///
    /// If a row sums to zero, it remains all zeros (the validator
    /// submitted no scores this epoch).
    pub fn normalize(&mut self) {
        for row in &mut self.weights {
            let sum: f64 = row.iter().sum();
            if sum > 0.0 {
                for w in row.iter_mut() {
                    *w /= sum;
                }
            }
        }
    }
}
