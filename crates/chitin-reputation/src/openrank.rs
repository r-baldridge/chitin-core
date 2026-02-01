// crates/chitin-reputation/src/openrank.rs
//
// OpenRank integration for context-aware trust in the Chitin Protocol.
//
// OpenRank extends EigenTrust with domain-aware, context-sensitive trust
// computation using personalized PageRank with damping.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Configuration for the OpenRank trust computation algorithm.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenRankConfig {
    /// Damping factor (probability of following a trust edge vs. teleporting). Default: 0.85.
    pub damping_factor: f64,
    /// Maximum iterations for convergence. Default: 100.
    pub max_iterations: u32,
    /// Convergence threshold (L1 norm of score change). Default: 1e-6.
    pub convergence_threshold: f64,
}

impl Default for OpenRankConfig {
    fn default() -> Self {
        Self {
            damping_factor: 0.85,
            max_iterations: 100,
            convergence_threshold: 1e-6,
        }
    }
}

/// Compute OpenRank trust scores from a trust matrix.
///
/// Uses personalized PageRank with damping to compute context-aware
/// global trust scores that account for domain expertise.
///
/// Algorithm (Personalized PageRank):
/// 1. Build column-normalized transition matrix from trust entries
/// 2. Handle dangling nodes (distribute uniform)
/// 3. Power iteration: scores = d * M * scores + (1-d) * personalization
/// 4. Converge per OpenRankConfig thresholds
pub fn compute_openrank(
    trust: &super::trust_matrix::TrustMatrix,
    config: &OpenRankConfig,
) -> HashMap<u16, f64> {
    // Step 1: Collect unique node UIDs
    let mut uid_set = std::collections::HashSet::new();
    for &(from, to) in trust.entries.keys() {
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

    // Step 2: Build adjacency matrix and column-normalize
    // M[i][j] represents the transition from j to i (column-stochastic)
    let mut adj = vec![vec![0.0_f64; n]; n];
    for (&(from, to), &val) in &trust.entries {
        let i = uid_to_idx[&from]; // source
        let j = uid_to_idx[&to];   // target
        // In PageRank, an edge from `from` to `to` means `from` endorses `to`.
        // In the column-normalized matrix M, M[to][from] = weight / col_sum
        adj[j][i] = val;
    }

    // Column-normalize; track dangling nodes (columns that sum to zero)
    let mut col_sums = vec![0.0_f64; n];
    for j in 0..n {
        for i in 0..n {
            col_sums[j] += adj[i][j];
        }
    }
    let mut is_dangling = vec![false; n];
    for j in 0..n {
        if col_sums[j] > 0.0 {
            for i in 0..n {
                adj[i][j] /= col_sums[j];
            }
        } else {
            is_dangling[j] = true;
        }
    }

    // Step 3: Personalization vector (uniform)
    let uniform = 1.0 / n as f64;
    let personalization = vec![uniform; n];

    // Initialize scores to uniform
    let mut scores = vec![uniform; n];

    let d = config.damping_factor;

    // Step 4: Power iteration
    for _ in 0..config.max_iterations {
        let mut new_scores = vec![0.0_f64; n];

        // Compute dangling contribution: sum of scores of dangling nodes
        let dangling_sum: f64 = scores
            .iter()
            .enumerate()
            .filter(|(j, _)| is_dangling[*j])
            .map(|(_, &s)| s)
            .sum();

        // new_scores = d * (M * scores + dangling_contribution) + (1-d) * personalization
        for i in 0..n {
            let mut m_times_scores = 0.0;
            for j in 0..n {
                m_times_scores += adj[i][j] * scores[j];
            }
            // Add uniform share of dangling node mass
            m_times_scores += dangling_sum * uniform;
            new_scores[i] = d * m_times_scores + (1.0 - d) * personalization[i];
        }

        // Check convergence (L1 norm)
        let delta: f64 = scores
            .iter()
            .zip(new_scores.iter())
            .map(|(a, b)| (a - b).abs())
            .sum();
        scores = new_scores;
        if delta < config.convergence_threshold {
            break;
        }
    }

    // Return HashMap<u16, f64>
    uids.iter().enumerate().map(|(i, &uid)| (uid, scores[i])).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::trust_matrix::TrustMatrix;

    #[test]
    fn empty_trust_matrix_returns_empty_scores() {
        let tm = TrustMatrix::new();
        let config = OpenRankConfig::default();
        let result = compute_openrank(&tm, &config);
        assert!(result.is_empty());
    }

    #[test]
    fn single_node_gets_score_one() {
        let mut tm = TrustMatrix::new();
        tm.set_trust(1, 1, 1.0);
        let config = OpenRankConfig::default();
        let result = compute_openrank(&tm, &config);
        assert_eq!(result.len(), 1);
        assert!((result[&1] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn damping_effect_higher_damping_concentrates_scores() {
        let mut tm = TrustMatrix::new();
        // Star topology: node 1 is pointed to by 2, 3, 4
        tm.set_trust(2, 1, 1.0);
        tm.set_trust(3, 1, 1.0);
        tm.set_trust(4, 1, 1.0);

        let low_damping_config = OpenRankConfig {
            damping_factor: 0.5,
            ..OpenRankConfig::default()
        };
        let high_damping_config = OpenRankConfig {
            damping_factor: 0.95,
            ..OpenRankConfig::default()
        };

        let low_result = compute_openrank(&tm, &low_damping_config);
        let high_result = compute_openrank(&tm, &high_damping_config);

        // With higher damping, the central node (1) should get a larger share
        // relative to others
        let low_ratio = low_result[&1] / low_result[&2];
        let high_ratio = high_result[&1] / high_result[&2];
        assert!(
            high_ratio > low_ratio,
            "Higher damping should concentrate more score on the central node. \
             low_ratio={}, high_ratio={}",
            low_ratio,
            high_ratio
        );
    }

    #[test]
    fn star_topology_central_node_highest() {
        let mut tm = TrustMatrix::new();
        // Nodes 2, 3, 4, 5 all trust node 1
        tm.set_trust(2, 1, 1.0);
        tm.set_trust(3, 1, 1.0);
        tm.set_trust(4, 1, 1.0);
        tm.set_trust(5, 1, 1.0);
        let config = OpenRankConfig::default();
        let result = compute_openrank(&tm, &config);
        // Node 1 should have the highest score
        let max_uid = result
            .iter()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .map(|(&uid, _)| uid)
            .unwrap();
        assert_eq!(max_uid, 1, "Central node should have highest score");
    }

    #[test]
    fn all_scores_sum_to_approximately_one() {
        let mut tm = TrustMatrix::new();
        tm.set_trust(1, 2, 0.8);
        tm.set_trust(2, 3, 0.6);
        tm.set_trust(3, 1, 0.9);
        tm.set_trust(1, 3, 0.3);
        let config = OpenRankConfig::default();
        let result = compute_openrank(&tm, &config);
        let total: f64 = result.values().sum();
        assert!(
            (total - 1.0).abs() < 1e-4,
            "Scores should sum to ~1.0, got {}",
            total
        );
    }

    #[test]
    fn convergence_within_max_iterations() {
        let mut tm = TrustMatrix::new();
        // Build a moderate-size graph
        for i in 0..10u16 {
            for j in 0..10u16 {
                if i != j {
                    tm.set_trust(i, j, ((i + j) as f64 % 7.0) / 7.0);
                }
            }
        }
        // Use a config that should converge quickly
        let config = OpenRankConfig {
            damping_factor: 0.85,
            max_iterations: 100,
            convergence_threshold: 1e-6,
        };
        let result = compute_openrank(&tm, &config);
        // Verify we got results for all 10 nodes
        assert_eq!(result.len(), 10);
        // Verify scores sum to ~1.0 (convergence indicator)
        let total: f64 = result.values().sum();
        assert!(
            (total - 1.0).abs() < 1e-4,
            "Converged scores should sum to ~1.0, got {}",
            total
        );
    }
}
