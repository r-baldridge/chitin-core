// crates/chitin-sync/src/range.rs
//
// Range-based sync for shard catchup in the Chitin Protocol.

use chitin_core::ChitinError;
use serde::{Deserialize, Serialize};

/// Range-based synchronization for catching up on missed epochs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RangeSync {
    /// First epoch to sync (inclusive).
    pub start_epoch: u64,
    /// Last epoch to sync (inclusive).
    pub end_epoch: u64,
}

impl RangeSync {
    /// Create a new RangeSync for the given epoch range.
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
    /// Validates that start_epoch <= end_epoch. Returns 0 because no peers
    /// are connected yet — the framework is ready for P2P integration.
    pub async fn sync_range(&self) -> Result<u64, ChitinError> {
        // Validate epoch range
        if self.start_epoch > self.end_epoch {
            return Err(ChitinError::InvalidState(format!(
                "Invalid epoch range: start {} > end {}",
                self.start_epoch, self.end_epoch
            )));
        }

        // No peers connected yet — return 0 synced Polyps.
        // Framework is ready for P2P integration.
        Ok(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn valid_range_returns_ok() {
        let sync = RangeSync::new(1, 10);
        let result = sync.sync_range().await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[tokio::test]
    async fn same_epoch_range_returns_ok() {
        let sync = RangeSync::new(5, 5);
        let result = sync.sync_range().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn inverted_range_returns_error() {
        let sync = RangeSync::new(10, 1);
        let result = sync.sync_range().await;
        assert!(result.is_err());
        match result.unwrap_err() {
            ChitinError::InvalidState(msg) => {
                assert!(msg.contains("start"));
                assert!(msg.contains("end"));
            }
            other => panic!("Expected InvalidState, got: {:?}", other),
        }
    }

    #[tokio::test]
    async fn zero_epoch_range_returns_ok() {
        let sync = RangeSync::new(0, 0);
        let result = sync.sync_range().await;
        assert!(result.is_ok());
    }
}
