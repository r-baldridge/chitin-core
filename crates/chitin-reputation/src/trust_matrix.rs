// crates/chitin-reputation/src/trust_matrix.rs
//
// Trust matrix: T_ij trust values per domain for the Chitin Protocol.
//
// Each entry T(from, to) represents how much node `from` trusts node `to`,
// based on historical scoring agreement and Polyp quality.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// A sparse trust matrix where T(from, to) = trust value.
///
/// Trust values range from 0.0 (no trust) to 1.0 (full trust).
/// The matrix is domain-scoped — a separate TrustMatrix can be
/// maintained for each Reef Zone.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustMatrix {
    /// Sparse trust entries: (from_uid, to_uid) -> trust_value.
    pub entries: HashMap<(u16, u16), f64>,
}

impl TrustMatrix {
    /// Create a new empty trust matrix.
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    /// Set the trust value from node `from` to node `to`.
    ///
    /// Values are clamped to [0.0, 1.0].
    pub fn set_trust(&mut self, from: u16, to: u16, value: f64) {
        let clamped = value.clamp(0.0, 1.0);
        self.entries.insert((from, to), clamped);
    }

    /// Get the trust value from node `from` to node `to`.
    ///
    /// Returns 0.0 if no trust relationship exists.
    pub fn get_trust(&self, from: u16, to: u16) -> f64 {
        self.entries.get(&(from, to)).copied().unwrap_or(0.0)
    }

    /// Compute global trust scores using EigenTrust-style iterative aggregation.
    ///
    /// Returns a map of node UID -> global trust score.
    ///
    /// # Phase 2
    /// This will implement the EigenTrust algorithm: iteratively aggregate
    /// local trust into global trust until convergence.
    pub fn compute_global_trust(&self) -> HashMap<u16, f64> {
        // Phase 2: Implement EigenTrust iterative trust aggregation
        todo!("Phase 2: TrustMatrix::compute_global_trust — EigenTrust iterative aggregation")
    }
}

impl Default for TrustMatrix {
    fn default() -> Self {
        Self::new()
    }
}
