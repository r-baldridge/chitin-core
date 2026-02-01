// crates/chitin-economics/src/rewards.rs
//
// Reward distribution logic for the Chitin Protocol.
//
// Each epoch's emission is split as follows:
//   1. TREASURY_FRACTION (2%) goes to the protocol treasury.
//   2. Remaining emission is split between Coral Nodes and Tide Nodes:
//      - Tide share:  VALIDATOR_FRACTION (xi = 0.41)
//      - Coral share: 1 - VALIDATOR_FRACTION (0.59)
//   3. Individual Coral rewards are proportional to their incentive scores.
//   4. Individual Tide rewards are proportional to their dividend scores.
//
// Reference: ARCHITECTURE.md Section 7.2

use std::collections::HashMap;

use crate::emission::{TREASURY_FRACTION, VALIDATOR_FRACTION};

/// The result of reward computation for a single epoch.
#[derive(Debug, Clone)]
pub struct RewardDistribution {
    /// Rewards for each Coral Node, keyed by UID (in rao).
    pub coral_rewards: HashMap<u16, u64>,
    /// Rewards for each Tide (validator) Node, keyed by UID (in rao).
    pub validator_rewards: HashMap<u16, u64>,
    /// Amount allocated to the protocol treasury (in rao).
    pub treasury_amount: u64,
}

/// Compute the reward distribution for an epoch.
///
/// # Arguments
/// - `epoch_emission_rao` — Total emission for this epoch in rao.
/// - `incentives` — Normalized incentive scores for Coral Nodes (should sum to ~1.0).
/// - `dividends` — Normalized dividend scores for Tide Nodes (should sum to ~1.0).
/// - `coral_uids` — UIDs of Coral Nodes, in the same order as `incentives`.
/// - `validator_uids` — UIDs of Tide Nodes, in the same order as `dividends`.
///
/// # Returns
/// A `RewardDistribution` with per-node rewards and treasury allocation.
///
/// # Panics
/// Panics if `incentives.len() != coral_uids.len()` or `dividends.len() != validator_uids.len()`.
pub fn compute_rewards(
    epoch_emission_rao: u64,
    incentives: &[f64],
    dividends: &[f64],
    coral_uids: &[u16],
    validator_uids: &[u16],
) -> RewardDistribution {
    assert_eq!(
        incentives.len(),
        coral_uids.len(),
        "incentives and coral_uids must have the same length"
    );
    assert_eq!(
        dividends.len(),
        validator_uids.len(),
        "dividends and validator_uids must have the same length"
    );

    // Step 1: Treasury allocation (2% of total emission)
    let treasury_amount = (epoch_emission_rao as f64 * TREASURY_FRACTION) as u64;
    let distributable = epoch_emission_rao - treasury_amount;

    // Step 2: Split between Coral (miners) and Tide (validators)
    let tide_pool = (distributable as f64 * VALIDATOR_FRACTION) as u64;
    let coral_pool = distributable - tide_pool;

    // Step 3: Distribute Coral rewards proportional to incentive scores
    let mut coral_rewards = HashMap::new();
    let incentive_sum: f64 = incentives.iter().sum();
    if incentive_sum > 0.0 {
        for (i, &uid) in coral_uids.iter().enumerate() {
            let share = incentives[i] / incentive_sum;
            let reward = (coral_pool as f64 * share) as u64;
            coral_rewards.insert(uid, reward);
        }
    }

    // Step 4: Distribute Tide rewards proportional to dividend scores
    let mut validator_rewards = HashMap::new();
    let dividend_sum: f64 = dividends.iter().sum();
    if dividend_sum > 0.0 {
        for (i, &uid) in validator_uids.iter().enumerate() {
            let share = dividends[i] / dividend_sum;
            let reward = (tide_pool as f64 * share) as u64;
            validator_rewards.insert(uid, reward);
        }
    }

    RewardDistribution {
        coral_rewards,
        validator_rewards,
        treasury_amount,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::token::RAO_PER_CTN;

    #[test]
    fn test_basic_reward_distribution() {
        let epoch_emission = 360 * RAO_PER_CTN; // 360 CTN for an epoch
        let incentives = vec![0.6, 0.4];
        let dividends = vec![0.5, 0.3, 0.2];
        let coral_uids = vec![0, 1];
        let validator_uids = vec![10, 11, 12];

        let dist = compute_rewards(
            epoch_emission,
            &incentives,
            &dividends,
            &coral_uids,
            &validator_uids,
        );

        // Treasury should be ~2% of total
        let expected_treasury = (epoch_emission as f64 * 0.02) as u64;
        assert_eq!(dist.treasury_amount, expected_treasury);

        // Total distributed should approximately equal epoch_emission - treasury
        let total_coral: u64 = dist.coral_rewards.values().sum();
        let total_validator: u64 = dist.validator_rewards.values().sum();

        // Allow for rounding errors (integer truncation)
        let total_distributed = total_coral + total_validator + dist.treasury_amount;
        assert!(total_distributed <= epoch_emission);
        // Should be close to the total (within a few rao of rounding)
        assert!(epoch_emission - total_distributed < 10);
    }

    #[test]
    fn test_treasury_fraction() {
        let epoch_emission = 1000 * RAO_PER_CTN;
        let dist = compute_rewards(epoch_emission, &[1.0], &[1.0], &[0], &[10]);

        let expected_treasury = (epoch_emission as f64 * TREASURY_FRACTION) as u64;
        assert_eq!(dist.treasury_amount, expected_treasury);
    }

    #[test]
    fn test_validator_fraction() {
        let epoch_emission = 1000 * RAO_PER_CTN;
        let dist = compute_rewards(epoch_emission, &[1.0], &[1.0], &[0], &[10]);

        let distributable = epoch_emission - dist.treasury_amount;
        let expected_validator = (distributable as f64 * VALIDATOR_FRACTION) as u64;
        assert_eq!(*dist.validator_rewards.get(&10).unwrap(), expected_validator);
    }

    #[test]
    fn test_empty_nodes() {
        let dist = compute_rewards(1000 * RAO_PER_CTN, &[], &[], &[], &[]);
        assert!(dist.coral_rewards.is_empty());
        assert!(dist.validator_rewards.is_empty());
        assert!(dist.treasury_amount > 0);
    }

    #[test]
    fn test_single_coral_gets_full_share() {
        let epoch_emission = 100 * RAO_PER_CTN;
        let dist = compute_rewards(epoch_emission, &[1.0], &[1.0], &[0], &[10]);

        let distributable = epoch_emission - dist.treasury_amount;
        let expected_coral = distributable - (distributable as f64 * VALIDATOR_FRACTION) as u64;
        assert_eq!(*dist.coral_rewards.get(&0).unwrap(), expected_coral);
    }
}
