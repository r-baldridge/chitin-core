// crates/chitin-consensus/src/epoch.rs
//
// Epoch lifecycle management for the Chitin Protocol.
//
// An epoch is a fixed-length period (default 360 blocks, ~1 hour) during which
// Tide Nodes evaluate Polyps, submit scores, and consensus is computed.
// Lifecycle: Open -> Scoring -> Committing -> Closed.

use serde::{Deserialize, Serialize};

/// The current phase of an epoch.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EpochPhase {
    /// Epoch is open: Coral Nodes submit Polyps, Tide Nodes begin evaluation.
    Open,
    /// Scoring phase: Tide Nodes are actively scoring Polyps.
    Scoring,
    /// Committing phase: Tide Nodes submit final weight vectors.
    Committing,
    /// Epoch is closed: Consensus has been computed, results are final.
    Closed,
}

/// Manages epoch transitions based on block height.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpochManager {
    /// The current epoch number.
    current_epoch: u64,
    /// The current phase within the epoch.
    phase: EpochPhase,
    /// Number of blocks per epoch (default 360).
    blocks_per_epoch: u64,
}

impl EpochManager {
    /// Create a new EpochManager.
    ///
    /// # Arguments
    /// * `blocks_per_epoch` - Number of blocks per epoch (e.g., 360 for ~1 hour at 10s/block).
    pub fn new(blocks_per_epoch: u64) -> Self {
        Self {
            current_epoch: 0,
            phase: EpochPhase::Open,
            blocks_per_epoch,
        }
    }

    /// Get the current epoch number.
    pub fn current_epoch(&self) -> u64 {
        self.current_epoch
    }

    /// Advance the epoch state based on the current block height.
    ///
    /// Computes the epoch number and phase from the absolute block height.
    /// Phase transitions occur at fixed fractions of the epoch:
    /// - Open: 0% - 50% of epoch blocks
    /// - Scoring: 50% - 75%
    /// - Committing: 75% - 100%
    /// - Closed: triggers epoch rollover
    pub fn advance_block(&mut self, block: u64) {
        let new_epoch = block / self.blocks_per_epoch;
        let block_in_epoch = block % self.blocks_per_epoch;

        self.current_epoch = new_epoch;

        // Determine phase based on position within epoch
        let fraction = block_in_epoch as f64 / self.blocks_per_epoch as f64;
        self.phase = if fraction < 0.50 {
            EpochPhase::Open
        } else if fraction < 0.75 {
            EpochPhase::Scoring
        } else {
            EpochPhase::Committing
        };
    }

    /// Get the current epoch phase.
    pub fn phase(&self) -> &EpochPhase {
        &self.phase
    }
}
