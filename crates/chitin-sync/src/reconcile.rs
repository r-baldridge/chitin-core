// crates/chitin-sync/src/reconcile.rs
//
// Set reconciliation: compute diff, request missing Polyps for the Chitin Protocol.
//
// After exchanging Vector Bloom Filters, nodes determine which Polyps
// the remote peer is missing and request them.

use chitin_core::ChitinError;
use uuid::Uuid;

use crate::vbf::VectorBloomFilter;

/// Manages set reconciliation between peers.
///
/// Compares a local VBF against a remote VBF (received as bytes)
/// to determine which Polyps need to be synced.
#[derive(Debug)]
pub struct SetReconciler {
    // Phase 2: Add P2P client handle for requesting missing Polyps
}

impl SetReconciler {
    /// Create a new SetReconciler.
    pub fn new() -> Self {
        Self {
            // Phase 2: Initialize with P2P client handle
        }
    }

    /// Compute the set difference between a local and remote VBF.
    ///
    /// Returns UUIDs of Polyps that are in the local set but probably
    /// not in the remote set (candidates for the remote to request).
    ///
    /// # Phase 2
    /// This will deserialize the remote VBF, iterate local Polyp IDs,
    /// and return those not present in the remote filter.
    pub fn compute_diff(
        &self,
        _local: &VectorBloomFilter,
        _remote: &[u8],
    ) -> Vec<Uuid> {
        // Phase 2: Deserialize remote VBF, iterate local IDs, return missing
        todo!("Phase 2: SetReconciler::compute_diff — compute set difference from VBFs")
    }

    /// Request missing Polyps from a remote peer.
    ///
    /// # Phase 2
    /// This will send a request to the remote peer via chitin-p2p
    /// for the specified Polyp IDs and store them locally.
    pub fn request_missing(
        &self,
        _missing: &[Uuid],
    ) -> Result<(), ChitinError> {
        // Phase 2: Request missing Polyps from remote peer via P2P
        todo!("Phase 2: SetReconciler::request_missing — fetch Polyps from remote peer")
    }
}

impl Default for SetReconciler {
    fn default() -> Self {
        Self::new()
    }
}
