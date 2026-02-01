// crates/chitin-store/src/hardened.rs
//
// HardenedStore: CID-indexed immutable Polyp storage.
//
// A hardened Polyp is one that has passed Yuma-Semantic Consensus, been
// IPFS-pinned, CID-anchored, and attested by validators. The HardenedStore
// provides:
//
//   - `store_hardened`: Serialize a Polyp, put it to IPFS (Phase 2), and
//     cache it locally in RocksDB under its CID key.
//   - `get_hardened`: Try local RocksDB cache first, then fall back to IPFS.
//   - `is_hardened`: Check whether a Polyp ID has a CID mapping recorded.
//
// Key format in RocksDB:
//   - `hardened:cid:{cid}` -> JSON-serialized Polyp
//   - `hardened:map:{polyp_uuid}` -> CID string (reverse lookup)

use uuid::Uuid;

use chitin_core::error::ChitinError;
use chitin_core::polyp::Polyp;

use crate::ipfs::IpfsClient;
use crate::rocks::RocksStore;

/// Store for CID-indexed, immutable (hardened) Polyps.
///
/// Wraps a local `RocksStore` (cache) and an `IpfsClient` (persistent
/// content-addressed storage). In Phase 1 the IPFS methods are stubs,
/// so only the local cache is functional.
#[derive(Debug)]
pub struct HardenedStore {
    /// Local RocksDB cache for fast CID-based lookups.
    pub local_cache: RocksStore,
    /// IPFS client for putting and retrieving hardened Polyps.
    pub ipfs: IpfsClient,
}

impl HardenedStore {
    /// Create a new `HardenedStore` backed by the given `RocksStore` and `IpfsClient`.
    pub fn new(local_cache: RocksStore, ipfs: IpfsClient) -> Self {
        Self { local_cache, ipfs }
    }

    /// Build the CID-based cache key: `hardened:cid:{cid}`.
    fn cid_key(cid: &str) -> Vec<u8> {
        format!("hardened:cid:{}", cid).into_bytes()
    }

    /// Build the reverse-lookup key: `hardened:map:{polyp_uuid}`.
    fn map_key(polyp_id: &Uuid) -> Vec<u8> {
        format!("hardened:map:{}", polyp_id).into_bytes()
    }

    /// Harden a Polyp: serialize it, put to IPFS (Phase 2), and cache locally.
    ///
    /// Returns the CID string assigned by IPFS. In Phase 1, the IPFS `put`
    /// call is a `todo!()` stub, so this method will panic if actually invoked.
    /// For Phase 1 local-only usage, callers should use `store_hardened_local`
    /// with a pre-computed or placeholder CID instead.
    pub fn store_hardened(&self, polyp: &Polyp) -> Result<String, ChitinError> {
        let json = serde_json::to_vec(polyp)
            .map_err(|e| ChitinError::Serialization(e.to_string()))?;

        // Phase 2: put to IPFS and get back a real CID.
        let cid = self.ipfs.put(&json);

        // Cache locally under the CID key.
        self.local_cache.put_bytes(&Self::cid_key(&cid), &json)?;

        // Record the polyp_id -> CID mapping for `is_hardened` lookups.
        self.local_cache
            .put_bytes(&Self::map_key(&polyp.id), cid.as_bytes())?;

        Ok(cid)
    }

    /// Store a hardened Polyp locally with a known CID (bypasses IPFS).
    ///
    /// Useful in Phase 1 where IPFS is not yet available, or when
    /// re-caching a Polyp whose CID is already known.
    pub fn store_hardened_local(&self, polyp: &Polyp, cid: &str) -> Result<(), ChitinError> {
        let json = serde_json::to_vec(polyp)
            .map_err(|e| ChitinError::Serialization(e.to_string()))?;

        self.local_cache.put_bytes(&Self::cid_key(cid), &json)?;
        self.local_cache
            .put_bytes(&Self::map_key(&polyp.id), cid.as_bytes())?;

        Ok(())
    }

    /// Retrieve a hardened Polyp by its CID.
    ///
    /// Tries the local RocksDB cache first. If not found, falls back to IPFS
    /// (Phase 2 stub — will panic if the local cache misses).
    pub fn get_hardened(&self, cid: &str) -> Result<Polyp, ChitinError> {
        // Try local cache first.
        if let Some(bytes) = self.local_cache.get_bytes(&Self::cid_key(cid))? {
            let polyp: Polyp = serde_json::from_slice(&bytes)
                .map_err(|e| ChitinError::Serialization(e.to_string()))?;
            return Ok(polyp);
        }

        // Fallback: fetch from IPFS (Phase 2 — will panic on todo!()).
        let bytes = self.ipfs.get_by_cid(cid);
        let polyp: Polyp = serde_json::from_slice(&bytes)
            .map_err(|e| ChitinError::Serialization(e.to_string()))?;

        // Cache locally for future lookups.
        self.local_cache.put_bytes(&Self::cid_key(cid), &bytes)?;

        Ok(polyp)
    }

    /// Check whether a given Polyp ID has been hardened (has a CID mapping).
    pub fn is_hardened(&self, polyp_id: Uuid) -> Result<bool, ChitinError> {
        let result = self.local_cache.get_bytes(&Self::map_key(&polyp_id))?;
        Ok(result.is_some())
    }
}
