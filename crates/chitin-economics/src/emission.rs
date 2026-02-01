// crates/chitin-economics/src/emission.rs
//
// Block reward emission schedule with halving.
//
// The Chitin Protocol uses a Bitcoin-like halving schedule:
// - Initial block reward: 1.0 CTN (= 10^9 rao)
// - Halving interval: 10,512,000 blocks (~4 years at 12s/block)
// - Treasury receives 2% of each block's emission
// - Validator fraction (xi): 0.41 of post-treasury emission
//
// Reference: ARCHITECTURE.md Sections 7.1, 7.2, 8.3

use crate::token::RAO_PER_CTN;

/// Number of blocks between each halving event.
/// 10,512,000 blocks at ~12 seconds per block is approximately 4 years.
pub const HALVING_INTERVAL: u64 = 10_512_000;

/// Initial block reward in rao: 1.0 CTN = 10^9 rao.
pub const INITIAL_BLOCK_REWARD_RAO: u64 = RAO_PER_CTN;

/// Fraction of each epoch's emission that goes to the protocol treasury (2%).
pub const TREASURY_FRACTION: f64 = 0.02;

/// Fraction of post-treasury emission allocated to Tide Nodes (validators).
/// xi = 0.41 means validators receive 41% and Coral Nodes receive 59%.
pub const VALIDATOR_FRACTION: f64 = 0.41;

/// Compute the block reward (in rao) at a given block height.
///
/// The reward halves every `HALVING_INTERVAL` blocks:
///   reward = INITIAL_BLOCK_REWARD_RAO / 2^(block / HALVING_INTERVAL)
///
/// Returns 0 when the halving number exceeds 63 (reward underflows to zero).
pub fn emission_at_block(block: u64) -> u64 {
    let halving_number = block / HALVING_INTERVAL;
    if halving_number >= 64 {
        // After 64 halvings, the reward is effectively zero
        return 0;
    }
    INITIAL_BLOCK_REWARD_RAO >> halving_number
}

/// Compute the total emission (in rao) for an epoch starting at `start_block`
/// that spans `blocks_per_epoch` blocks.
///
/// This handles the case where a halving occurs mid-epoch by summing
/// block-by-block rewards across the halving boundary.
pub fn epoch_emission(start_block: u64, blocks_per_epoch: u64) -> u64 {
    let end_block = start_block + blocks_per_epoch;

    // Optimization: if the entire epoch falls within a single halving period,
    // we can multiply rather than iterate.
    let start_halving = start_block / HALVING_INTERVAL;
    let end_halving = (end_block.saturating_sub(1)) / HALVING_INTERVAL;

    if start_halving == end_halving {
        // All blocks in this epoch have the same reward
        let reward = emission_at_block(start_block);
        return reward * blocks_per_epoch;
    }

    // Halving boundary crosses the epoch — sum segment by segment
    let mut total: u64 = 0;
    let mut current_block = start_block;

    while current_block < end_block {
        let current_halving = current_block / HALVING_INTERVAL;
        let next_halving_block = (current_halving + 1) * HALVING_INTERVAL;
        let segment_end = end_block.min(next_halving_block);
        let blocks_in_segment = segment_end - current_block;
        let reward = emission_at_block(current_block);
        total = total.saturating_add(reward * blocks_in_segment);
        current_block = segment_end;
    }

    total
}

/// Compute the cumulative total emission (in rao) from block 0 through block `blocks - 1`.
///
/// This is the total supply that has been emitted up to (but not including) the given block.
pub fn cumulative_emission(blocks: u64) -> u64 {
    if blocks == 0 {
        return 0;
    }

    let mut total: u64 = 0;
    let mut remaining = blocks;
    let mut halving_number: u64 = 0;

    while remaining > 0 && halving_number < 64 {
        let blocks_in_this_halving = if halving_number == blocks / HALVING_INTERVAL {
            // This is the current (potentially partial) halving period
            blocks - halving_number * HALVING_INTERVAL
        } else {
            remaining.min(HALVING_INTERVAL)
        };

        let reward = INITIAL_BLOCK_REWARD_RAO >> halving_number;
        if reward == 0 {
            break;
        }
        total = total.saturating_add(reward * blocks_in_this_halving);
        remaining -= blocks_in_this_halving;
        halving_number += 1;
    }

    total
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_emission_at_block_zero() {
        assert_eq!(emission_at_block(0), RAO_PER_CTN); // 1.0 CTN
    }

    #[test]
    fn test_emission_before_first_halving() {
        assert_eq!(emission_at_block(1_000_000), RAO_PER_CTN);
        assert_eq!(emission_at_block(HALVING_INTERVAL - 1), RAO_PER_CTN);
    }

    #[test]
    fn test_emission_at_first_halving() {
        assert_eq!(emission_at_block(HALVING_INTERVAL), RAO_PER_CTN / 2);
    }

    #[test]
    fn test_emission_at_second_halving() {
        assert_eq!(emission_at_block(HALVING_INTERVAL * 2), RAO_PER_CTN / 4);
    }

    #[test]
    fn test_emission_at_very_high_block() {
        // After 64 halvings, reward should be 0
        assert_eq!(emission_at_block(HALVING_INTERVAL * 64), 0);
    }

    #[test]
    fn test_epoch_emission_no_halving_boundary() {
        // 360 blocks within the first halving period
        let emission = epoch_emission(0, 360);
        assert_eq!(emission, 360 * RAO_PER_CTN);
    }

    #[test]
    fn test_epoch_emission_at_halving_boundary() {
        // Epoch that spans the first halving boundary
        let start = HALVING_INTERVAL - 100;
        let blocks = 200;
        let emission = epoch_emission(start, blocks);
        // 100 blocks at full reward + 100 blocks at half reward
        let expected = 100 * RAO_PER_CTN + 100 * (RAO_PER_CTN / 2);
        assert_eq!(emission, expected);
    }

    #[test]
    fn test_cumulative_emission_zero() {
        assert_eq!(cumulative_emission(0), 0);
    }

    #[test]
    fn test_cumulative_emission_one_block() {
        assert_eq!(cumulative_emission(1), RAO_PER_CTN);
    }

    #[test]
    fn test_cumulative_emission_first_halving_period() {
        let total = cumulative_emission(HALVING_INTERVAL);
        assert_eq!(total, HALVING_INTERVAL * RAO_PER_CTN);
    }
}
