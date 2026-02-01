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
        // Step 1: Collect unique node UIDs
        let mut uid_set = std::collections::HashSet::new();
        for &(from, to) in self.entries.keys() {
            uid_set.insert(from);
            uid_set.insert(to);
        }
        if uid_set.is_empty() {
            return HashMap::new();
        }
        let mut uids: Vec<u16> = uid_set.into_iter().collect();
        uids.sort();
        let n = uids.len();
        let uid_to_idx: HashMap<u16, usize> = uids.iter().enumerate().map(|(i, &u)| (u, i)).collect();

        // Step 2: Build row-normalized trust matrix C[i][j]
        // C[i][j] = local_trust(i,j) / sum_k(local_trust(i,k))
        let mut c = vec![vec![0.0_f64; n]; n];
        for (&(from, to), &val) in &self.entries {
            let i = uid_to_idx[&from];
            let j = uid_to_idx[&to];
            c[i][j] = val;
        }
        // Row-normalize; if row is all zeros, use uniform
        for i in 0..n {
            let row_sum: f64 = c[i].iter().sum();
            if row_sum > 0.0 {
                for j in 0..n {
                    c[i][j] /= row_sum;
                }
            } else {
                // Uniform distribution for dangling nodes
                for j in 0..n {
                    c[i][j] = 1.0 / n as f64;
                }
            }
        }

        // Step 3: Initialize trust vector t = uniform 1/n
        let uniform = 1.0 / n as f64;
        let mut t = vec![uniform; n];

        // Pre-trust vector p = uniform 1/n
        let p = vec![uniform; n];
        let alpha = 0.1; // pre-trust weight

        // Step 4: Iterate until convergence
        // t_new[j] = (1 - alpha) * sum_i(C[i][j] * t[i]) + alpha * p[j]
        let max_iter = 100;
        let epsilon = 1e-8;

        for _ in 0..max_iter {
            let mut t_new = vec![0.0_f64; n];
            for j in 0..n {
                let mut sum = 0.0;
                for i in 0..n {
                    sum += c[i][j] * t[i];
                }
                t_new[j] = (1.0 - alpha) * sum + alpha * p[j];
            }

            // Check convergence: L1 delta
            let delta: f64 = t.iter().zip(t_new.iter()).map(|(a, b)| (a - b).abs()).sum();
            t = t_new;
            if delta < epsilon {
                break;
            }
        }

        // Step 5: Return HashMap<u16, f64>
        uids.iter().enumerate().map(|(i, &uid)| (uid, t[i])).collect()
    }
}

impl Default for TrustMatrix {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_matrix_returns_empty_map() {
        let tm = TrustMatrix::new();
        let result = tm.compute_global_trust();
        assert!(result.is_empty());
    }

    #[test]
    fn self_trust_single_node() {
        let mut tm = TrustMatrix::new();
        tm.set_trust(1, 1, 1.0);
        let result = tm.compute_global_trust();
        assert_eq!(result.len(), 1);
        assert!((result[&1] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn mutual_trust_two_nodes() {
        let mut tm = TrustMatrix::new();
        tm.set_trust(1, 2, 1.0);
        tm.set_trust(2, 1, 1.0);
        let result = tm.compute_global_trust();
        assert_eq!(result.len(), 2);
        // Symmetric trust should give equal scores
        assert!((result[&1] - result[&2]).abs() < 1e-6);
        // Each should be ~0.5
        assert!((result[&1] - 0.5).abs() < 1e-4);
    }

    #[test]
    fn chain_trust_c_gets_higher_score() {
        // A trusts B, B trusts C
        let mut tm = TrustMatrix::new();
        tm.set_trust(1, 2, 1.0); // A -> B
        tm.set_trust(2, 3, 1.0); // B -> C
        let result = tm.compute_global_trust();
        assert_eq!(result.len(), 3);
        // C should get higher global trust than A because trust flows A->B->C
        assert!(
            result[&3] > result[&1],
            "C ({}) should have higher trust than A ({})",
            result[&3],
            result[&1]
        );
    }

    #[test]
    fn convergence_scores_sum_to_one() {
        let mut tm = TrustMatrix::new();
        tm.set_trust(1, 2, 0.8);
        tm.set_trust(2, 3, 0.6);
        tm.set_trust(3, 1, 0.9);
        tm.set_trust(1, 3, 0.3);
        let result = tm.compute_global_trust();
        let total: f64 = result.values().sum();
        assert!(
            (total - 1.0).abs() < 1e-4,
            "Scores should sum to ~1.0, got {}",
            total
        );
    }

    #[test]
    fn sybil_resistance_untrusted_sybils_get_low_scores() {
        let mut tm = TrustMatrix::new();
        // Honest nodes form a well-connected cluster
        tm.set_trust(1, 2, 1.0);
        tm.set_trust(2, 1, 1.0);
        tm.set_trust(1, 3, 1.0);
        tm.set_trust(3, 1, 1.0);
        tm.set_trust(2, 3, 1.0);
        tm.set_trust(3, 2, 1.0);
        // Sybil cluster: 10, 11, 12 all trust each other
        tm.set_trust(10, 11, 1.0);
        tm.set_trust(11, 10, 1.0);
        tm.set_trust(10, 12, 1.0);
        tm.set_trust(12, 10, 1.0);
        tm.set_trust(11, 12, 1.0);
        tm.set_trust(12, 11, 1.0);
        // Sybils try to gain trust by trusting honest nodes, but honest nodes don't trust back
        tm.set_trust(10, 1, 1.0);
        tm.set_trust(11, 1, 1.0);
        tm.set_trust(12, 1, 1.0);
        let result = tm.compute_global_trust();
        // In EigenTrust, trust flows from sybils toward honest nodes but not back.
        // The honest cluster retains its trust while sybils leak trust out.
        // Per-node, honest nodes should have higher global trust scores.
        let honest_score = result[&1] + result[&2] + result[&3];
        let sybil_score = result[&10] + result[&11] + result[&12];
        assert!(
            honest_score > sybil_score,
            "Honest cluster total trust ({}) should exceed sybil cluster total trust ({})",
            honest_score,
            sybil_score
        );
    }
}
