// crates/chitin-store/src/rocks.rs
//
// RocksDB-backed persistent storage for Polyps.
//
// Key format:
//   - Primary:   `polyp:{uuid}` -> JSON-serialized Polyp
//   - Secondary: `state:{state_tag}:{uuid}` -> empty value (index only)
//
// The secondary index allows efficient listing of Polyps by lifecycle state
// without scanning the entire keyspace.

use async_trait::async_trait;
use rocksdb::{DBWithThreadMode, MultiThreaded, Options};
use uuid::Uuid;

use chitin_core::error::ChitinError;
use chitin_core::polyp::{Polyp, PolypState};
use chitin_core::traits::PolypStore;

/// RocksDB wrapper implementing the `PolypStore` trait.
#[derive(Debug)]
pub struct RocksStore {
    db: DBWithThreadMode<MultiThreaded>,
}

impl RocksStore {
    /// Open a RocksDB database at the given filesystem path.
    ///
    /// Creates the database directory if it does not exist.
    pub fn open(path: &str) -> Result<Self, ChitinError> {
        let mut opts = Options::default();
        opts.create_if_missing(true);

        let db = DBWithThreadMode::<MultiThreaded>::open(&opts, path)
            .map_err(|e| ChitinError::Storage(format!("Failed to open RocksDB at {}: {}", path, e)))?;

        Ok(Self { db })
    }

    /// Build the primary key for a Polyp: `polyp:{uuid}`.
    fn polyp_key(id: &Uuid) -> Vec<u8> {
        format!("polyp:{}", id).into_bytes()
    }

    /// Build the secondary index key: `state:{tag}:{uuid}`.
    fn state_key(state: &PolypState, id: &Uuid) -> Vec<u8> {
        format!("state:{}:{}", state_tag(state), id).into_bytes()
    }

    /// Put raw bytes into RocksDB, mapping errors to ChitinError::Storage.
    fn put_raw(&self, key: &[u8], value: &[u8]) -> Result<(), ChitinError> {
        self.db
            .put(key, value)
            .map_err(|e| ChitinError::Storage(format!("RocksDB put failed: {}", e)))
    }

    /// Get raw bytes from RocksDB, mapping errors to ChitinError::Storage.
    fn get_raw(&self, key: &[u8]) -> Result<Option<Vec<u8>>, ChitinError> {
        self.db
            .get(key)
            .map_err(|e| ChitinError::Storage(format!("RocksDB get failed: {}", e)))
    }

    /// Delete a key from RocksDB, mapping errors to ChitinError::Storage.
    fn delete_raw(&self, key: &[u8]) -> Result<(), ChitinError> {
        self.db
            .delete(key)
            .map_err(|e| ChitinError::Storage(format!("RocksDB delete failed: {}", e)))
    }

    /// Low-level: store a Polyp with its primary key and secondary state index entry.
    fn store_polyp_inner(&self, polyp: &Polyp) -> Result<(), ChitinError> {
        let json = serde_json::to_vec(polyp)?;
        self.put_raw(&Self::polyp_key(&polyp.id), &json)?;
        // Write secondary state index (empty value — existence is the signal).
        self.put_raw(&Self::state_key(&polyp.state, &polyp.id), &[])?;
        Ok(())
    }

    /// Low-level: remove the secondary state index entry for a Polyp.
    fn remove_state_index(&self, state: &PolypState, id: &Uuid) -> Result<(), ChitinError> {
        self.delete_raw(&Self::state_key(state, id))
    }

    /// Public accessor: get a Polyp by UUID without going through the async trait.
    /// Useful for internal callers (e.g., `HardenedStore`) that already hold a reference.
    pub fn get_polyp_sync(&self, id: &Uuid) -> Result<Option<Polyp>, ChitinError> {
        match self.get_raw(&Self::polyp_key(id))? {
            Some(bytes) => {
                let polyp: Polyp = serde_json::from_slice(&bytes)?;
                Ok(Some(polyp))
            }
            None => Ok(None),
        }
    }

    /// Public accessor: store a Polyp synchronously.
    pub fn save_polyp_sync(&self, polyp: &Polyp) -> Result<(), ChitinError> {
        // If the Polyp already exists, remove the old state index entry
        // before writing the new one (the state may have changed).
        if let Some(existing) = self.get_polyp_sync(&polyp.id)? {
            if existing.state != polyp.state {
                self.remove_state_index(&existing.state, &polyp.id)?;
            }
        }
        self.store_polyp_inner(polyp)
    }

    /// Store a value under an arbitrary key. Used by `HardenedStore` for CID-indexed entries.
    pub fn put_bytes(&self, key: &[u8], value: &[u8]) -> Result<(), ChitinError> {
        self.put_raw(key, value)
    }

    /// Retrieve a value by arbitrary key. Used by `HardenedStore` for CID-indexed lookups.
    pub fn get_bytes(&self, key: &[u8]) -> Result<Option<Vec<u8>>, ChitinError> {
        self.get_raw(key)
    }
}

#[async_trait]
impl PolypStore for RocksStore {
    async fn save_polyp(&self, polyp: &Polyp) -> Result<(), ChitinError> {
        self.save_polyp_sync(polyp)
    }

    async fn get_polyp(&self, id: &Uuid) -> Result<Option<Polyp>, ChitinError> {
        self.get_polyp_sync(id)
    }

    async fn list_polyps_by_state(&self, state: &PolypState) -> Result<Vec<Polyp>, ChitinError> {
        let prefix_str = format!("state:{}:", state_tag(state));
        let prefix = prefix_str.as_bytes();
        let mut polyps = Vec::new();

        let iter = self.db.prefix_iterator(prefix);
        for item in iter {
            let (key, _value) = item
                .map_err(|e| ChitinError::Storage(format!("RocksDB iteration error: {}", e)))?;

            // Keys are `state:{tag}:{uuid}`. Stop when the prefix no longer matches.
            if !key.starts_with(prefix) {
                break;
            }

            // Extract the UUID from the key suffix (bytes after the prefix).
            let uuid_bytes = &key[prefix.len()..];
            let uuid_str = std::str::from_utf8(uuid_bytes).unwrap_or("");
            if let Ok(id) = Uuid::parse_str(uuid_str) {
                if let Some(polyp) = self.get_polyp_sync(&id)? {
                    polyps.push(polyp);
                }
            }
        }

        Ok(polyps)
    }

    async fn delete_polyp(&self, id: &Uuid) -> Result<(), ChitinError> {
        // Remove the state index entry first, if the Polyp exists.
        if let Some(existing) = self.get_polyp_sync(id)? {
            self.remove_state_index(&existing.state, id)?;
        }
        self.delete_raw(&Self::polyp_key(id))
    }
}

/// Convert a `PolypState` to a short string tag for use in secondary index keys.
///
/// This avoids relying on `Display` or `Debug` which might include variant data
/// (e.g., `Molted { successor_id: ... }`). We use a stable, compact tag instead.
fn state_tag(state: &PolypState) -> &'static str {
    match state {
        PolypState::Draft => "draft",
        PolypState::Soft => "soft",
        PolypState::UnderReview => "under_review",
        PolypState::Approved => "approved",
        PolypState::Hardened => "hardened",
        PolypState::Rejected => "rejected",
        PolypState::Molted { .. } => "molted",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_tag_values() {
        assert_eq!(state_tag(&PolypState::Draft), "draft");
        assert_eq!(state_tag(&PolypState::Soft), "soft");
        assert_eq!(state_tag(&PolypState::UnderReview), "under_review");
        assert_eq!(state_tag(&PolypState::Approved), "approved");
        assert_eq!(state_tag(&PolypState::Hardened), "hardened");
        assert_eq!(state_tag(&PolypState::Rejected), "rejected");
        assert_eq!(
            state_tag(&PolypState::Molted {
                successor_id: Uuid::nil()
            }),
            "molted"
        );
    }
}
