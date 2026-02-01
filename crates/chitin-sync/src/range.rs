// crates/chitin-sync/src/range.rs
//
// Range-based sync for shard catchup in the Chitin Protocol.
//
// When a node joins or falls behind, it uses range sync to fetch
// all Polyps from a range of epochs from peers.

use chitin_core::ChitinError;
use serde::{Deserialize, Serialize};

/// Range-based synchronization for catching up on missed epochs.
///
/// Fetches all Polyps hardened during a contiguous range of epochs,
/// enabling new nodes or nodes that were offline to catch up to
/// the current state of the Reef.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RangeSync {
    /// First epoch to sync (inclusive).
    pub start_epoch: u64,
    /// Last epoch to sync (inclusive).
    pub end_epoch: u64,
}

impl RangeSync {
    /// Create a new RangeSync for the given epoch range.
    ///
    /// # Arguments
    /// * `start` - First epoch to sync (inclusive).
    /// * `end` - Last epoch to sync (inclusive).
    pub fn new(start: u64, end: u64) -> Self {
        Self {
            start_epoch: start,
            end_epoch: end,
        }
    }

    /// Execute the range sync, fetching all Polyps from the specified epochs.
    ///
    /// Returns the total number of Polyps synced.
    ///
    /// # Phase 2
    /// This will:
    /// 1. Query peers for epoch summaries in the range
    /// 2. Download hardened Polyps for each epoch
    /// 3. Verify Merkle proofs and attestations
    /// 4. Store synced Polyps in the local store
    pub async fn sync_range(&self) -> Result<u64, ChitinError> {
        // Phase 2: Fetch, verify, and store Polyps from epoch range
        todo!("Phase 2: RangeSync::sync_range — fetch Polyps from epoch range")
    }
}
