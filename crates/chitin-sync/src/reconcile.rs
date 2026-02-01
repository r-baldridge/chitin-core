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
/// Compares local Polyp IDs against a remote VBF (received as bytes)
/// to determine which Polyps need to be synced.
#[derive(Debug)]
pub struct SetReconciler {
    /// The set of local Polyp IDs known to this node.
    local_ids: Vec<Uuid>,
}

impl SetReconciler {
    /// Create a new SetReconciler with no local IDs.
    pub fn new() -> Self {
        Self {
            local_ids: Vec::new(),
        }
    }

    /// Create a new SetReconciler initialized with the given local IDs.
    pub fn with_local_ids(ids: Vec<Uuid>) -> Self {
        Self { local_ids: ids }
    }

    /// Set (replace) the local IDs for reconciliation.
    pub fn set_local_ids(&mut self, ids: Vec<Uuid>) {
        self.local_ids = ids;
    }

    /// Compute the set difference between local IDs and a remote VBF.
    ///
    /// Deserializes the remote VBF from bytes, then checks each local ID
    /// against it. Returns the IDs that are NOT in the remote filter
    /// (i.e., the Polyps the remote peer is missing and needs to receive).
    pub fn compute_diff(
        &self,
        _local: &VectorBloomFilter,
        remote: &[u8],
    ) -> Result<Vec<Uuid>, ChitinError> {
        let remote_vbf = VectorBloomFilter::from_bytes(remote)?;
        let missing: Vec<Uuid> = self
            .local_ids
            .iter()
            .filter(|id| !remote_vbf.contains(id))
            .copied()
            .collect();
        Ok(missing)
    }

    /// Request missing Polyps from a remote peer.
    ///
    /// # Phase 2 (incomplete)
    /// This will send a request to the remote peer via chitin-p2p
    /// for the specified Polyp IDs and store them locally.
    /// Requires a P2P client handle which is not yet available.
    pub fn request_missing(
        &self,
        missing: &[Uuid],
    ) -> Result<(), ChitinError> {
        if missing.is_empty() {
            return Ok(());
        }

        // No P2P connection available yet — return error for non-empty requests.
        Err(ChitinError::Network(
            "no P2P connection available".to_string(),
        ))
    }
}

impl Default for SetReconciler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_local_ids_missing_from_empty_remote() {
        let ids: Vec<Uuid> = (0..5).map(|_| Uuid::now_v7()).collect();
        let reconciler = SetReconciler::with_local_ids(ids.clone());

        // Create an empty remote VBF (nothing inserted)
        let remote_vbf = VectorBloomFilter::new(100);
        let remote_bytes = remote_vbf.to_bytes();

        let local_vbf = VectorBloomFilter::new(100);
        let missing = reconciler
            .compute_diff(&local_vbf, &remote_bytes)
            .expect("compute_diff should succeed");

        // All local IDs should be missing from the empty remote
        assert_eq!(missing.len(), ids.len());
        for id in &ids {
            assert!(missing.contains(id));
        }
    }

    #[test]
    fn none_missing_when_all_in_remote() {
        let ids: Vec<Uuid> = (0..5).map(|_| Uuid::now_v7()).collect();
        let reconciler = SetReconciler::with_local_ids(ids.clone());

        // Insert all local IDs into the remote VBF
        let mut remote_vbf = VectorBloomFilter::new(100);
        for id in &ids {
            remote_vbf.insert(id);
        }
        let remote_bytes = remote_vbf.to_bytes();

        let local_vbf = VectorBloomFilter::new(100);
        let missing = reconciler
            .compute_diff(&local_vbf, &remote_bytes)
            .expect("compute_diff should succeed");

        assert!(
            missing.is_empty(),
            "No IDs should be missing when all are in remote, got {} missing",
            missing.len()
        );
    }

    #[test]
    fn partial_overlap() {
        let shared: Vec<Uuid> = (0..3).map(|_| Uuid::now_v7()).collect();
        let local_only: Vec<Uuid> = (0..2).map(|_| Uuid::now_v7()).collect();

        let mut all_local = shared.clone();
        all_local.extend_from_slice(&local_only);
        let reconciler = SetReconciler::with_local_ids(all_local);

        // Remote only has the shared IDs
        let mut remote_vbf = VectorBloomFilter::new(100);
        for id in &shared {
            remote_vbf.insert(id);
        }
        let remote_bytes = remote_vbf.to_bytes();

        let local_vbf = VectorBloomFilter::new(100);
        let missing = reconciler
            .compute_diff(&local_vbf, &remote_bytes)
            .expect("compute_diff should succeed");

        // The local-only IDs should be in the missing set
        for id in &local_only {
            assert!(
                missing.contains(id),
                "Local-only ID should be reported as missing"
            );
        }
        // Shared IDs should NOT be in the missing set
        for id in &shared {
            assert!(
                !missing.contains(id),
                "Shared ID should not be reported as missing"
            );
        }
    }

    #[test]
    fn invalid_remote_bytes_returns_error() {
        let ids: Vec<Uuid> = (0..3).map(|_| Uuid::now_v7()).collect();
        let reconciler = SetReconciler::with_local_ids(ids);

        let invalid_bytes = vec![0u8; 10];
        let local_vbf = VectorBloomFilter::new(100);
        let result = reconciler.compute_diff(&local_vbf, &invalid_bytes);

        assert!(result.is_err(), "Should return error for invalid remote bytes");
    }

    #[test]
    fn request_missing_empty_ok() {
        let reconciler = SetReconciler::new();
        let result = reconciler.request_missing(&[]);
        assert!(result.is_ok());
    }

    #[test]
    fn request_missing_nonempty_returns_network_error() {
        let reconciler = SetReconciler::new();
        let ids = vec![Uuid::now_v7(), Uuid::now_v7()];
        let result = reconciler.request_missing(&ids);
        assert!(result.is_err());
        match result.unwrap_err() {
            ChitinError::Network(msg) => {
                assert!(msg.contains("no P2P connection"));
            }
            other => panic!("Expected Network error, got: {:?}", other),
        }
    }
}
