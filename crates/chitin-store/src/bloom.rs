// crates/chitin-store/src/bloom.rs
//
// Bloom filter for efficient probabilistic Polyp ID membership testing.
//
// Used during set reconciliation (chitin-sync) to quickly determine which
// Polyps a peer has that we are missing, without exchanging full ID lists.
// The Bloom filter trades a small false-positive rate for massive bandwidth
// savings when comparing large Polyp sets.

use bloomfilter::Bloom;
use uuid::Uuid;

/// Bloom filter wrapping Polyp UUIDs for fast set membership testing.
///
/// False positives are possible (a UUID may appear to be present when it is not),
/// but false negatives are impossible (if `contains` returns `false`, the UUID
/// is definitely not in the set).
#[derive(Debug, Clone)]
pub struct PolypBloomFilter {
    inner: Bloom<String>,
}

impl PolypBloomFilter {
    /// Create a new Bloom filter sized for the given expected item count
    /// and desired false positive rate.
    ///
    /// # Arguments
    ///
    /// * `expected_items` - Estimated number of Polyp IDs to insert.
    /// * `false_positive_rate` - Target FP rate, e.g. 0.01 for 1%.
    ///
    /// # Panics
    ///
    /// Panics if `expected_items` is 0 or `false_positive_rate` is not in (0, 1).
    pub fn new(expected_items: usize, false_positive_rate: f64) -> Self {
        assert!(expected_items > 0, "expected_items must be > 0");
        assert!(
            false_positive_rate > 0.0 && false_positive_rate < 1.0,
            "false_positive_rate must be in (0, 1)"
        );

        let inner = Bloom::new_for_fp_rate(expected_items, false_positive_rate);
        Self { inner }
    }

    /// Insert a Polyp UUID into the Bloom filter.
    pub fn insert(&mut self, polyp_id: &Uuid) {
        self.inner.set(&polyp_id.to_string());
    }

    /// Check whether a Polyp UUID might be in the set.
    ///
    /// Returns `true` if the UUID is *probably* in the set (subject to the
    /// configured false positive rate). Returns `false` if the UUID is
    /// *definitely not* in the set.
    pub fn contains(&self, polyp_id: &Uuid) -> bool {
        self.inner.check(&polyp_id.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_and_contains() {
        let mut bf = PolypBloomFilter::new(100, 0.01);
        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();

        bf.insert(&id1);

        assert!(bf.contains(&id1));
        // id2 was not inserted — it should (almost certainly) not be found.
        // With fp_rate=0.01 and only 1 element, the probability of a false
        // positive for a single check is very low.
        // We do not assert !contains(id2) because false positives are possible,
        // but in practice this test will pass.
        let _ = bf.contains(&id2);
    }

    #[test]
    fn test_multiple_inserts() {
        let mut bf = PolypBloomFilter::new(1000, 0.01);
        let ids: Vec<Uuid> = (0..100).map(|_| Uuid::new_v4()).collect();

        for id in &ids {
            bf.insert(id);
        }

        // All inserted IDs must be found (no false negatives).
        for id in &ids {
            assert!(bf.contains(id), "False negative for UUID {}", id);
        }
    }

    #[test]
    #[should_panic(expected = "expected_items must be > 0")]
    fn test_zero_items_panics() {
        let _ = PolypBloomFilter::new(0, 0.01);
    }
}
