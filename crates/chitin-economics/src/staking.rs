// crates/chitin-economics/src/staking.rs
//
// Stake management: stake/unstake logic with minimum requirements and cooldown periods.
//
// Node types have different minimum stake requirements (in CTN):
//   - Coral Node: 100 CTN
//   - Tide Node: 1,000 CTN
//   - Delegation: 10 CTN
//
// Unstaking is subject to a cooldown period (in blocks):
//   - Coral Node: 7,200 blocks (~24 hours)
//   - Tide Node: 21,600 blocks (~72 hours)
//   - Delegation: 7,200 blocks (~24 hours)
//
// Reference: ARCHITECTURE.md Section 7.3, configs/economics.yaml

use serde::{Deserialize, Serialize};

use crate::token::RAO_PER_CTN;
use chitin_core::error::ChitinError;

/// Minimum stake for a Coral Node: 100 CTN (in rao).
pub const CORAL_MINIMUM: u64 = 100 * RAO_PER_CTN;

/// Minimum stake for a Tide Node: 1,000 CTN (in rao).
pub const TIDE_MINIMUM: u64 = 1_000 * RAO_PER_CTN;

/// Minimum stake for delegation: 10 CTN (in rao).
pub const DELEGATION_MINIMUM: u64 = 10 * RAO_PER_CTN;

/// Cooldown period for Coral Node unstaking: 7,200 blocks (~24 hours at 12s/block).
pub const CORAL_COOLDOWN_BLOCKS: u64 = 7_200;

/// Cooldown period for Tide Node unstaking: 21,600 blocks (~72 hours at 12s/block).
pub const TIDE_COOLDOWN_BLOCKS: u64 = 21_600;

/// Cooldown period for delegation unstaking: 7,200 blocks (~24 hours at 12s/block).
pub const DELEGATION_COOLDOWN_BLOCKS: u64 = 7_200;

/// A single stake entry representing a staker's commitment to a node.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakeEntry {
    /// The coldkey of the staker (owner of the funds).
    pub staker: [u8; 32],
    /// Amount staked in rao.
    pub amount: u64,
    /// The network UID of the node being staked to.
    pub node_uid: u16,
    /// Block number at which the stake was created.
    pub staked_at_block: u64,
    /// If set, the block at which unstaking was requested. The actual unstake
    /// completes after the cooldown period elapses from this block.
    pub unstake_requested_at: Option<u64>,
}

/// Manages all stake entries for the network.
///
/// Provides operations for staking, requesting unstakes, and processing
/// completed cooldown periods.
pub struct StakeManager {
    entries: Vec<StakeEntry>,
}

impl StakeManager {
    /// Create a new empty StakeManager.
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Add a new stake entry.
    ///
    /// Validates that the stake amount meets the specified minimum.
    /// The `minimum` parameter should be one of: `CORAL_MINIMUM`, `TIDE_MINIMUM`,
    /// or `DELEGATION_MINIMUM`, depending on the node type being staked to.
    ///
    /// # Errors
    /// Returns `ChitinError::InvalidState` if the stake amount is below the minimum.
    pub fn stake(&mut self, entry: StakeEntry) -> Result<(), ChitinError> {
        // Validate minimum stake — caller is responsible for choosing the right minimum
        // based on node type. We check against the delegation minimum as a baseline.
        if entry.amount < DELEGATION_MINIMUM {
            return Err(ChitinError::InvalidState(format!(
                "Stake amount {} rao is below the minimum delegation requirement of {} rao ({} CTN)",
                entry.amount,
                DELEGATION_MINIMUM,
                DELEGATION_MINIMUM / RAO_PER_CTN
            )));
        }

        self.entries.push(entry);
        Ok(())
    }

    /// Request unstaking for a given staker and node.
    ///
    /// Marks the stake entry with the current block number so the cooldown
    /// period can be tracked. The stake remains locked until the cooldown
    /// elapses and `process_unstakes` is called.
    ///
    /// # Errors
    /// Returns `ChitinError::NotFound` if no matching active stake entry is found.
    /// Returns `ChitinError::InvalidState` if the entry already has a pending unstake.
    pub fn request_unstake(
        &mut self,
        staker: &[u8; 32],
        node_uid: u16,
        current_block: u64,
    ) -> Result<(), ChitinError> {
        let entry = self
            .entries
            .iter_mut()
            .find(|e| e.staker == *staker && e.node_uid == node_uid && e.unstake_requested_at.is_none())
            .ok_or_else(|| {
                ChitinError::NotFound(format!(
                    "No active stake entry found for staker and node_uid {}",
                    node_uid
                ))
            })?;

        if entry.unstake_requested_at.is_some() {
            return Err(ChitinError::InvalidState(
                "Unstake already requested for this entry".to_string(),
            ));
        }

        entry.unstake_requested_at = Some(current_block);
        Ok(())
    }

    /// Process all unstake requests that have completed their cooldown period.
    ///
    /// Returns the list of `StakeEntry` values that have been fully unstaked
    /// and removes them from the manager.
    ///
    /// The `cooldown_blocks` parameter specifies how many blocks must elapse
    /// after the unstake request before funds are released. Use the appropriate
    /// constant (`CORAL_COOLDOWN_BLOCKS`, `TIDE_COOLDOWN_BLOCKS`, or
    /// `DELEGATION_COOLDOWN_BLOCKS`) based on the node type.
    ///
    /// For simplicity in Phase 1, this uses a single cooldown value for all entries.
    /// Phase 2+ should differentiate by node type.
    pub fn process_unstakes(&mut self, current_block: u64) -> Vec<StakeEntry> {
        let mut completed = Vec::new();
        let mut remaining = Vec::new();

        for entry in self.entries.drain(..) {
            if let Some(requested_at) = entry.unstake_requested_at {
                // Phase 1: Use the coral cooldown as a conservative default.
                // Phase 2+: Look up cooldown based on node type.
                let cooldown = CORAL_COOLDOWN_BLOCKS;
                if current_block >= requested_at + cooldown {
                    completed.push(entry);
                } else {
                    remaining.push(entry);
                }
            } else {
                remaining.push(entry);
            }
        }

        self.entries = remaining;
        completed
    }

    /// Compute the total stake (in rao) for a given node UID.
    ///
    /// Only counts active stakes (not those with a pending unstake request).
    pub fn total_stake_for_node(&self, node_uid: u16) -> u64 {
        self.entries
            .iter()
            .filter(|e| e.node_uid == node_uid && e.unstake_requested_at.is_none())
            .map(|e| e.amount)
            .sum()
    }

    /// Get all stake entries (for inspection/debugging).
    pub fn entries(&self) -> &[StakeEntry] {
        &self.entries
    }
}

impl Default for StakeManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_staker() -> [u8; 32] {
        [1u8; 32]
    }

    fn make_entry(amount: u64, node_uid: u16, block: u64) -> StakeEntry {
        StakeEntry {
            staker: test_staker(),
            amount,
            node_uid,
            staked_at_block: block,
            unstake_requested_at: None,
        }
    }

    #[test]
    fn test_stake_above_minimum() {
        let mut manager = StakeManager::new();
        let entry = make_entry(CORAL_MINIMUM, 0, 100);
        assert!(manager.stake(entry).is_ok());
        assert_eq!(manager.entries().len(), 1);
    }

    #[test]
    fn test_stake_below_minimum() {
        let mut manager = StakeManager::new();
        let entry = make_entry(DELEGATION_MINIMUM - 1, 0, 100);
        assert!(manager.stake(entry).is_err());
    }

    #[test]
    fn test_total_stake_for_node() {
        let mut manager = StakeManager::new();
        manager
            .stake(make_entry(CORAL_MINIMUM, 0, 100))
            .unwrap();
        manager
            .stake(make_entry(CORAL_MINIMUM * 2, 0, 200))
            .unwrap();
        manager
            .stake(make_entry(CORAL_MINIMUM, 1, 100))
            .unwrap();

        assert_eq!(manager.total_stake_for_node(0), CORAL_MINIMUM * 3);
        assert_eq!(manager.total_stake_for_node(1), CORAL_MINIMUM);
        assert_eq!(manager.total_stake_for_node(99), 0);
    }

    #[test]
    fn test_request_unstake() {
        let mut manager = StakeManager::new();
        manager
            .stake(make_entry(CORAL_MINIMUM, 0, 100))
            .unwrap();
        assert!(manager
            .request_unstake(&test_staker(), 0, 500)
            .is_ok());
        assert_eq!(manager.entries()[0].unstake_requested_at, Some(500));
    }

    #[test]
    fn test_request_unstake_not_found() {
        let mut manager = StakeManager::new();
        assert!(manager
            .request_unstake(&test_staker(), 0, 500)
            .is_err());
    }

    #[test]
    fn test_process_unstakes_before_cooldown() {
        let mut manager = StakeManager::new();
        manager
            .stake(make_entry(CORAL_MINIMUM, 0, 100))
            .unwrap();
        manager.request_unstake(&test_staker(), 0, 500).unwrap();

        // Process before cooldown elapses
        let completed = manager.process_unstakes(500 + CORAL_COOLDOWN_BLOCKS - 1);
        assert!(completed.is_empty());
        assert_eq!(manager.entries().len(), 1);
    }

    #[test]
    fn test_process_unstakes_after_cooldown() {
        let mut manager = StakeManager::new();
        manager
            .stake(make_entry(CORAL_MINIMUM, 0, 100))
            .unwrap();
        manager.request_unstake(&test_staker(), 0, 500).unwrap();

        // Process after cooldown elapses
        let completed = manager.process_unstakes(500 + CORAL_COOLDOWN_BLOCKS);
        assert_eq!(completed.len(), 1);
        assert_eq!(completed[0].amount, CORAL_MINIMUM);
        assert!(manager.entries().is_empty());
    }

    #[test]
    fn test_unstaked_node_excluded_from_total() {
        let mut manager = StakeManager::new();
        manager
            .stake(make_entry(CORAL_MINIMUM, 0, 100))
            .unwrap();
        manager.request_unstake(&test_staker(), 0, 500).unwrap();

        // Pending unstake should not count toward total
        assert_eq!(manager.total_stake_for_node(0), 0);
    }
}
