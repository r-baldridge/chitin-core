// crates/chitin-consensus/src/metagraph.rs
//
// Metagraph state management for the Chitin Protocol.
//
// The ReefMetagraph is the global network state: all nodes, stakes, trust scores,
// weights, bonds, and Polyp counts. Updated every epoch.

use chitin_core::{ChitinError, ReefMetagraph};

/// Manages the local view of the Reef Metagraph.
///
/// Each node maintains a local copy of the metagraph that is updated
/// every epoch with the latest consensus results.
#[derive(Debug)]
pub struct MetagraphManager {
    /// The current metagraph snapshot.
    current: Option<ReefMetagraph>,
    /// The last epoch number seen (for monotonicity validation).
    last_epoch: Option<u64>,
}

impl MetagraphManager {
    /// Create a new MetagraphManager with no initial metagraph.
    pub fn new() -> Self {
        Self {
            current: None,
            last_epoch: None,
        }
    }

    /// Update the local metagraph with a new snapshot.
    ///
    /// Validates epoch monotonicity: the new metagraph's epoch must be
    /// strictly greater than the last seen epoch.
    pub fn update(&mut self, metagraph: ReefMetagraph) -> Result<(), ChitinError> {
        if let Some(last) = self.last_epoch {
            if metagraph.epoch <= last {
                return Err(ChitinError::Consensus(format!(
                    "Stale epoch: got {} but last was {}",
                    metagraph.epoch, last
                )));
            }
        }
        self.last_epoch = Some(metagraph.epoch);
        self.current = Some(metagraph);
        Ok(())
    }

    /// Get a reference to the current metagraph snapshot, if available.
    pub fn current(&self) -> Option<&ReefMetagraph> {
        self.current.as_ref()
    }
}

impl Default for MetagraphManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn make_metagraph(epoch: u64) -> ReefMetagraph {
        ReefMetagraph {
            epoch,
            block: epoch * 100,
            nodes: vec![],
            total_stake: 0,
            total_hardened_polyps: 0,
            emission_rate: 0,
            weights: HashMap::new(),
            bonds: HashMap::new(),
        }
    }

    #[test]
    fn test_new_manager_has_no_current() {
        let manager = MetagraphManager::new();
        assert!(manager.current().is_none());
    }

    #[test]
    fn test_update_stores_metagraph() {
        let mut manager = MetagraphManager::new();
        let mg = make_metagraph(1);
        manager.update(mg).unwrap();

        let current = manager.current().expect("should have current metagraph");
        assert_eq!(current.epoch, 1);
    }

    #[test]
    fn test_update_rejects_stale_epoch() {
        let mut manager = MetagraphManager::new();

        // First update with epoch 5
        manager.update(make_metagraph(5)).unwrap();

        // Try to update with same epoch — should fail
        let result = manager.update(make_metagraph(5));
        assert!(result.is_err());

        // Try to update with lower epoch — should fail
        let result = manager.update(make_metagraph(3));
        assert!(result.is_err());

        // Current should still be epoch 5
        assert_eq!(manager.current().unwrap().epoch, 5);
    }

    #[test]
    fn test_update_accepts_higher_epoch() {
        let mut manager = MetagraphManager::new();

        manager.update(make_metagraph(1)).unwrap();
        assert_eq!(manager.current().unwrap().epoch, 1);

        manager.update(make_metagraph(5)).unwrap();
        assert_eq!(manager.current().unwrap().epoch, 5);

        manager.update(make_metagraph(100)).unwrap();
        assert_eq!(manager.current().unwrap().epoch, 100);
    }
}
