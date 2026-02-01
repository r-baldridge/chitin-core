// crates/chitin-consensus/src/bonds.rs
//
// Bond matrix management and penalty computation for the Chitin Protocol.
//
// Bonds represent a validator's historical commitment to scoring a specific
// Coral Node. Built up via EMA over epochs. Higher bonds = higher dividend share.

use serde::{Deserialize, Serialize};

use crate::weights::WeightMatrix;

/// A dense bond matrix where B[validator_idx][coral_idx] = bond value.
///
/// Bonds are EMA-smoothed historical weights that represent a validator's
/// long-term commitment to evaluating a Coral Node. Bonds decay when
/// validators disagree with consensus (bond penalty).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BondMatrix {
    /// Dense bond matrix: bonds[validator_idx][coral_idx].
    pub bonds: Vec<Vec<f64>>,
}

impl BondMatrix {
    /// Create a new zero-initialized bond matrix.
    ///
    /// # Arguments
    /// * `validators` - Number of validators (Tide Nodes).
    /// * `corals` - Number of Coral Nodes.
    pub fn new(validators: usize, corals: usize) -> Self {
        Self {
            bonds: vec![vec![0.0; corals]; validators],
        }
    }

    /// Update bonds using EMA with penalty for consensus deviation.
    ///
    /// For each (validator, coral) pair:
    ///   ema = alpha * W[i][j] + (1 - alpha) * B_prev[i][j]
    ///   penalty = bond_penalty * |W[i][j] - consensus_weight[j]|
    ///   B[i][j] = max(0.0, ema - penalty)
    ///
    /// # Phase 3
    /// This will implement the bond EMA update with penalty from
    /// ARCHITECTURE.md Section 4.2, Step 5.
    pub fn update_ema(
        &mut self,
        weights: &WeightMatrix,
        alpha: f64,
        bond_penalty: f64,
        consensus_weights: &[f64],
    ) {
        let num_validators = self.bonds.len();
        for i in 0..num_validators {
            let num_corals = self.bonds[i].len();
            for j in 0..num_corals {
                let w_ij = weights.weights[i][j];
                let b_prev = self.bonds[i][j];
                let consensus_j = if j < consensus_weights.len() {
                    consensus_weights[j]
                } else {
                    0.0
                };
                let ema = alpha * w_ij + (1.0 - alpha) * b_prev;
                let penalty = bond_penalty * (w_ij - consensus_j).abs();
                self.bonds[i][j] = (ema - penalty).max(0.0);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_ema_without_penalty() {
        let mut bonds = BondMatrix::new(2, 2);
        // Set some previous bonds
        bonds.bonds[0][0] = 0.5;
        bonds.bonds[0][1] = 0.3;
        bonds.bonds[1][0] = 0.4;
        bonds.bonds[1][1] = 0.6;

        let mut weights = WeightMatrix::new(2, 2);
        weights.set(0, 0, 0.8);
        weights.set(0, 1, 0.2);
        weights.set(1, 0, 0.6);
        weights.set(1, 1, 0.4);

        let alpha = 0.3;
        let consensus = vec![0.8, 0.2]; // matches validator 0 exactly

        // bond_penalty = 0 means no penalty
        bonds.update_ema(&weights, alpha, 0.0, &consensus);

        // B[0][0] = 0.3 * 0.8 + 0.7 * 0.5 = 0.24 + 0.35 = 0.59
        assert!((bonds.bonds[0][0] - 0.59).abs() < 1e-10);
        // B[0][1] = 0.3 * 0.2 + 0.7 * 0.3 = 0.06 + 0.21 = 0.27
        assert!((bonds.bonds[0][1] - 0.27).abs() < 1e-10);
        // B[1][0] = 0.3 * 0.6 + 0.7 * 0.4 = 0.18 + 0.28 = 0.46
        assert!((bonds.bonds[1][0] - 0.46).abs() < 1e-10);
        // B[1][1] = 0.3 * 0.4 + 0.7 * 0.6 = 0.12 + 0.42 = 0.54
        assert!((bonds.bonds[1][1] - 0.54).abs() < 1e-10);
    }

    #[test]
    fn test_ema_with_penalty_reduces_bonds_for_disagreeing_validators() {
        let mut bonds = BondMatrix::new(2, 1);
        bonds.bonds[0][0] = 0.5;
        bonds.bonds[1][0] = 0.5;

        let mut weights = WeightMatrix::new(2, 1);
        weights.set(0, 0, 0.8); // agrees with consensus
        weights.set(1, 0, 0.2); // disagrees with consensus

        let alpha = 0.5;
        let bond_penalty = 0.5;
        let consensus = vec![0.8];

        bonds.update_ema(&weights, alpha, bond_penalty, &consensus);

        // Validator 0 (agrees): ema = 0.5*0.8 + 0.5*0.5 = 0.65, penalty = 0.5*|0.8-0.8| = 0.0
        // B[0][0] = 0.65
        assert!((bonds.bonds[0][0] - 0.65).abs() < 1e-10);

        // Validator 1 (disagrees): ema = 0.5*0.2 + 0.5*0.5 = 0.35, penalty = 0.5*|0.2-0.8| = 0.3
        // B[1][0] = 0.35 - 0.3 = 0.05
        assert!((bonds.bonds[1][0] - 0.05).abs() < 1e-10);

        // Disagreeing validator has lower bond
        assert!(bonds.bonds[0][0] > bonds.bonds[1][0]);
    }

    #[test]
    fn test_bonds_clamp_to_zero() {
        let mut bonds = BondMatrix::new(1, 1);
        bonds.bonds[0][0] = 0.1;

        let mut weights = WeightMatrix::new(1, 1);
        weights.set(0, 0, 0.0); // far from consensus

        let alpha = 0.5;
        let bond_penalty = 2.0; // very large penalty
        let consensus = vec![1.0];

        bonds.update_ema(&weights, alpha, bond_penalty, &consensus);

        // ema = 0.5*0.0 + 0.5*0.1 = 0.05, penalty = 2.0*|0.0-1.0| = 2.0
        // B = max(0, 0.05 - 2.0) = 0.0
        assert_eq!(bonds.bonds[0][0], 0.0);
    }

    #[test]
    fn test_empty_matrix_stays_empty() {
        let mut bonds = BondMatrix::new(0, 0);
        let weights = WeightMatrix::new(0, 0);
        let consensus: Vec<f64> = vec![];

        bonds.update_ema(&weights, 0.5, 0.1, &consensus);

        assert!(bonds.bonds.is_empty());
    }
}
