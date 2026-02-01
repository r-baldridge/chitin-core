// crates/chitin-sync/src/vbf.rs
//
// Vector Bloom Filter construction and exchange for the Chitin Protocol.
//
// A Vector Bloom Filter (VBF) is a compact probabilistic set summary
// used to efficiently determine which Polyps a remote peer is missing.
// Nodes exchange VBFs and compute set differences to identify
// Polyps that need to be synchronized.

use bloomfilter::Bloom;
use chitin_core::ChitinError;
use uuid::Uuid;

/// A Vector Bloom Filter wrapping a probabilistic set membership structure.
///
/// Used for efficient set reconciliation between peers. Each node
/// inserts its known Polyp IDs into a VBF and exchanges it with peers.
/// The receiving peer checks its own IDs against the remote VBF to
/// identify Polyps the remote is missing.
pub struct VectorBloomFilter {
    /// The underlying Bloom filter.
    inner: Bloom<[u8; 16]>,
}

impl std::fmt::Debug for VectorBloomFilter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VectorBloomFilter")
            .field("inner", &"<Bloom>")
            .finish()
    }
}

impl VectorBloomFilter {
    /// Create a new VectorBloomFilter with the given expected capacity.
    ///
    /// Uses a false positive rate of 0.01 (1%) which provides a good
    /// balance between filter size and accuracy for set reconciliation.
    pub fn new(capacity: usize) -> Self {
        let bloom = Bloom::new_for_fp_rate(capacity, 0.01);
        Self { inner: bloom }
    }

    /// Insert a Polyp UUID into the Bloom filter.
    pub fn insert(&mut self, id: &Uuid) {
        self.inner.set(&id.into_bytes());
    }

    /// Check whether a Polyp UUID is probably in the filter.
    ///
    /// Returns `true` if the ID is probably present (may be a false positive).
    /// Returns `false` if the ID is definitely not present.
    pub fn contains(&self, id: &Uuid) -> bool {
        self.inner.check(&id.into_bytes())
    }

    /// Serialize the Bloom filter to bytes for network exchange.
    ///
    /// Binary format:
    /// ```text
    /// [8 bytes: bitmap_bits u64 LE]
    /// [4 bytes: k_num u32 LE]
    /// [8 bytes: sip_key_0.0 u64 LE]
    /// [8 bytes: sip_key_0.1 u64 LE]
    /// [8 bytes: sip_key_1.0 u64 LE]
    /// [8 bytes: sip_key_1.1 u64 LE]
    /// [rest: bitmap bytes]
    /// ```
    /// 44-byte header + variable-length bitmap.
    pub fn to_bytes(&self) -> Vec<u8> {
        let bitmap = self.inner.bitmap();
        let bitmap_bits = self.inner.number_of_bits();
        let k_num = self.inner.number_of_hash_functions();
        let sip_keys = self.inner.sip_keys();

        let mut buf = Vec::with_capacity(44 + bitmap.len());
        buf.extend_from_slice(&bitmap_bits.to_le_bytes());
        buf.extend_from_slice(&k_num.to_le_bytes());
        buf.extend_from_slice(&sip_keys[0].0.to_le_bytes());
        buf.extend_from_slice(&sip_keys[0].1.to_le_bytes());
        buf.extend_from_slice(&sip_keys[1].0.to_le_bytes());
        buf.extend_from_slice(&sip_keys[1].1.to_le_bytes());
        buf.extend_from_slice(&bitmap);
        buf
    }

    /// Deserialize a Bloom filter from bytes received from a peer.
    ///
    /// Returns an error if the data is too short (< 44 bytes header).
    pub fn from_bytes(data: &[u8]) -> Result<Self, ChitinError> {
        const HEADER_SIZE: usize = 44;
        if data.len() < HEADER_SIZE {
            return Err(ChitinError::Serialization(format!(
                "VBF data too short: expected at least {} bytes, got {}",
                HEADER_SIZE,
                data.len()
            )));
        }

        let bitmap_bits = u64::from_le_bytes(
            data[0..8]
                .try_into()
                .map_err(|e| ChitinError::Serialization(format!("Failed to read bitmap_bits: {}", e)))?,
        );
        let k_num = u32::from_le_bytes(
            data[8..12]
                .try_into()
                .map_err(|e| ChitinError::Serialization(format!("Failed to read k_num: {}", e)))?,
        );
        let sip_key_0_0 = u64::from_le_bytes(
            data[12..20]
                .try_into()
                .map_err(|e| ChitinError::Serialization(format!("Failed to read sip_key: {}", e)))?,
        );
        let sip_key_0_1 = u64::from_le_bytes(
            data[20..28]
                .try_into()
                .map_err(|e| ChitinError::Serialization(format!("Failed to read sip_key: {}", e)))?,
        );
        let sip_key_1_0 = u64::from_le_bytes(
            data[28..36]
                .try_into()
                .map_err(|e| ChitinError::Serialization(format!("Failed to read sip_key: {}", e)))?,
        );
        let sip_key_1_1 = u64::from_le_bytes(
            data[36..44]
                .try_into()
                .map_err(|e| ChitinError::Serialization(format!("Failed to read sip_key: {}", e)))?,
        );

        let sip_keys = [(sip_key_0_0, sip_key_0_1), (sip_key_1_0, sip_key_1_1)];
        let bitmap_bytes = &data[HEADER_SIZE..];

        let bloom = Bloom::from_existing(bitmap_bytes, bitmap_bits, k_num, sip_keys);
        Ok(VectorBloomFilter { inner: bloom })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_empty_filter() {
        let vbf = VectorBloomFilter::new(100);
        let bytes = vbf.to_bytes();
        let restored = VectorBloomFilter::from_bytes(&bytes).expect("deserialization should succeed");

        // An empty filter should not contain any random UUID
        let id = Uuid::now_v7();
        assert!(!restored.contains(&id));
    }

    #[test]
    fn roundtrip_with_items() {
        let mut vbf = VectorBloomFilter::new(100);
        let id1 = Uuid::now_v7();
        let id2 = Uuid::now_v7();
        let id3 = Uuid::now_v7();
        vbf.insert(&id1);
        vbf.insert(&id2);
        vbf.insert(&id3);

        let bytes = vbf.to_bytes();
        let restored = VectorBloomFilter::from_bytes(&bytes).expect("deserialization should succeed");

        assert!(restored.contains(&id1));
        assert!(restored.contains(&id2));
        assert!(restored.contains(&id3));

        // An ID not inserted should (almost certainly) not be present
        let id_absent = Uuid::now_v7();
        assert!(!restored.contains(&id_absent));
    }

    #[test]
    fn from_bytes_too_short_returns_error() {
        let short_data = vec![0u8; 10];
        let result = VectorBloomFilter::from_bytes(&short_data);
        assert!(result.is_err());
        let err = result.unwrap_err();
        let msg = format!("{}", err);
        assert!(msg.contains("too short"), "Error message should mention 'too short', got: {}", msg);
    }

    #[test]
    fn roundtrip_100_items_preserved() {
        let mut vbf = VectorBloomFilter::new(200);
        let ids: Vec<Uuid> = (0..100).map(|_| Uuid::now_v7()).collect();
        for id in &ids {
            vbf.insert(id);
        }

        let bytes = vbf.to_bytes();
        let restored = VectorBloomFilter::from_bytes(&bytes).expect("deserialization should succeed");

        for id in &ids {
            assert!(
                restored.contains(id),
                "Restored filter should contain all inserted IDs"
            );
        }
    }

    #[test]
    fn false_positive_rate_preserved_after_roundtrip() {
        let item_count = 1000;
        let mut vbf = VectorBloomFilter::new(item_count);
        let ids: Vec<Uuid> = (0..item_count).map(|_| Uuid::now_v7()).collect();
        for id in &ids {
            vbf.insert(id);
        }

        let bytes = vbf.to_bytes();
        let restored = VectorBloomFilter::from_bytes(&bytes).expect("deserialization should succeed");

        // Test false positive rate with IDs not in the filter
        let test_count = 10_000;
        let mut false_positives = 0;
        for _ in 0..test_count {
            let test_id = Uuid::now_v7();
            if restored.contains(&test_id) {
                false_positives += 1;
            }
        }

        let fp_rate = false_positives as f64 / test_count as f64;
        // The configured FP rate is 0.01; allow some tolerance (up to 0.05)
        assert!(
            fp_rate < 0.05,
            "False positive rate {} is too high after roundtrip (expected < 0.05)",
            fp_rate
        );
    }
}
