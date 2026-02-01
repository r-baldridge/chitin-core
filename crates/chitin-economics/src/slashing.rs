// crates/chitin-economics/src/slashing.rs
//
// Slashing conditions and penalty computation for the Chitin Protocol.
//
// Four conditions trigger slashing (partial or full stake forfeiture):
//   1. Invalid ZK Proof — 100% of stake (critical)
//   2. Consensus Deviation — 5% of stake per offense (moderate)
//   3. Liveness Failure — 1% of stake per missed epoch (low)
//   4. Duplicate Submission — 10% of stake (moderate)
//
// Slashed tokens flow to the protocol treasury.
//
// Reference: ARCHITECTURE.md Section 7.4, configs/economics.yaml

use serde::{Deserialize, Serialize};

/// Slash rate for submitting an invalid ZK proof: 100% of stake.
pub const INVALID_ZK_PROOF_RATE: f64 = 1.0;

/// Slash rate for consensus deviation: 5% of stake per offense.
pub const CONSENSUS_DEVIATION_RATE: f64 = 0.05;

/// Slash rate for liveness failure: 1% of stake per missed epoch.
pub const LIVENESS_FAILURE_RATE: f64 = 0.01;

/// Slash rate for duplicate submission: 10% of stake.
pub const DUPLICATE_SUBMISSION_RATE: f64 = 0.10;

/// Conditions that trigger slashing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SlashCondition {
    /// Coral Node submitted a Polyp with a ZK proof that does not verify.
    /// Indicates dishonest embedding generation. Severity: Critical.
    InvalidZkProof,

    /// Tide Node consistently scores in strong disagreement with consensus
    /// (>3 sigma deviation for 3+ consecutive epochs).
    /// Indicates collusion or incompetence. Severity: Moderate.
    ConsensusDeviation,

    /// Node fails to respond to ValidationQueries or submit weights
    /// for 3+ consecutive epochs. Severity: Low.
    LivenessFailure,

    /// Coral Node submits a Polyp that is a near-duplicate (cosine similarity > 0.98)
    /// of an existing hardened Polyp in the same model namespace. Severity: Moderate.
    DuplicateSubmission,
}

/// Result of a slashing event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlashResult {
    /// The condition that triggered the slash.
    pub condition: SlashCondition,
    /// The coldkey of the offending node.
    pub offender: [u8; 32],
    /// The amount of stake slashed (in rao).
    pub amount_slashed: u64,
}

/// Compute the penalty amount (in rao) for a given slashing condition.
///
/// # Arguments
/// - `condition` — The type of offense.
/// - `current_stake` — The offender's current total stake in rao.
///
/// # Returns
/// The penalty amount in rao. Never exceeds `current_stake`.
pub fn compute_penalty(condition: &SlashCondition, current_stake: u64) -> u64 {
    let rate = match condition {
        SlashCondition::InvalidZkProof => INVALID_ZK_PROOF_RATE,
        SlashCondition::ConsensusDeviation => CONSENSUS_DEVIATION_RATE,
        SlashCondition::LivenessFailure => LIVENESS_FAILURE_RATE,
        SlashCondition::DuplicateSubmission => DUPLICATE_SUBMISSION_RATE,
    };

    let penalty = (current_stake as f64 * rate) as u64;
    // Ensure penalty does not exceed current stake
    penalty.min(current_stake)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::token::RAO_PER_CTN;

    #[test]
    fn test_invalid_zk_proof_slashes_all() {
        let stake = 100 * RAO_PER_CTN;
        let penalty = compute_penalty(&SlashCondition::InvalidZkProof, stake);
        assert_eq!(penalty, stake); // 100% slash
    }

    #[test]
    fn test_consensus_deviation_slashes_5_percent() {
        let stake = 1000 * RAO_PER_CTN;
        let penalty = compute_penalty(&SlashCondition::ConsensusDeviation, stake);
        let expected = (stake as f64 * 0.05) as u64;
        assert_eq!(penalty, expected);
    }

    #[test]
    fn test_liveness_failure_slashes_1_percent() {
        let stake = 1000 * RAO_PER_CTN;
        let penalty = compute_penalty(&SlashCondition::LivenessFailure, stake);
        let expected = (stake as f64 * 0.01) as u64;
        assert_eq!(penalty, expected);
    }

    #[test]
    fn test_duplicate_submission_slashes_10_percent() {
        let stake = 500 * RAO_PER_CTN;
        let penalty = compute_penalty(&SlashCondition::DuplicateSubmission, stake);
        let expected = (stake as f64 * 0.10) as u64;
        assert_eq!(penalty, expected);
    }

    #[test]
    fn test_penalty_does_not_exceed_stake() {
        // Even with 100% rate, penalty should not exceed stake
        let stake = 50 * RAO_PER_CTN;
        let penalty = compute_penalty(&SlashCondition::InvalidZkProof, stake);
        assert!(penalty <= stake);
    }

    #[test]
    fn test_zero_stake() {
        let penalty = compute_penalty(&SlashCondition::InvalidZkProof, 0);
        assert_eq!(penalty, 0);
    }
}
