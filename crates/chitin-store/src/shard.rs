// crates/chitin-store/src/shard.rs
//
// Consistent-hash shard assignment for Polyp distribution.
//
// Phase 1: Runs single-shard (num_shards=1), so all Polyps land on shard 0.
// Phase 2+: Multi-shard deployment where Polyps are distributed across
// multiple storage/index nodes based on a deterministic hash of their UUID.
//
// The hash function uses the UUID's raw bytes to compute a stable shard
// assignment. This ensures that the same Polyp ID always maps to the same
// shard, regardless of which node computes the assignment.

use uuid::Uuid;

/// Assigns Polyp IDs to shards using a simple hash-based scheme.
///
/// In Phase 1 this is configured with `num_shards=1`, meaning all Polyps
/// are stored on a single node. In Phase 2+, multi-shard deployment
/// distributes Polyps across `num_shards` storage/index partitions.
#[derive(Debug, Clone)]
pub struct ShardAssigner {
    /// Total number of shards in the system.
    num_shards: u16,
}

impl ShardAssigner {
    /// Create a new shard assigner with the given number of shards.
    ///
    /// # Panics
    ///
    /// Panics if `num_shards` is 0.
    pub fn new(num_shards: u16) -> Self {
        assert!(num_shards > 0, "num_shards must be > 0");
        Self { num_shards }
    }

    /// Deterministically assign a Polyp UUID to a shard index in `[0, num_shards)`.
    ///
    /// Uses a simple FNV-1a-inspired hash of the UUID bytes to produce a
    /// uniformly distributed shard assignment. The result is stable: the same
    /// UUID always maps to the same shard for a given `num_shards`.
    pub fn assign_shard(&self, polyp_id: &Uuid) -> u16 {
        let bytes = polyp_id.as_bytes();
        let mut hash: u64 = 0xcbf29ce484222325; // FNV-1a offset basis
        for &byte in bytes {
            hash ^= byte as u64;
            hash = hash.wrapping_mul(0x100000001b3); // FNV-1a prime
        }
        (hash % self.num_shards as u64) as u16
    }

    /// Return the total number of shards.
    pub fn num_shards(&self) -> u16 {
        self.num_shards
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_shard_always_zero() {
        let assigner = ShardAssigner::new(1);
        for _ in 0..100 {
            let id = Uuid::new_v4();
            assert_eq!(assigner.assign_shard(&id), 0);
        }
    }

    #[test]
    fn test_deterministic_assignment() {
        let assigner = ShardAssigner::new(16);
        let id = Uuid::new_v4();
        let shard1 = assigner.assign_shard(&id);
        let shard2 = assigner.assign_shard(&id);
        assert_eq!(shard1, shard2, "Same UUID must always map to same shard");
    }

    #[test]
    fn test_shard_within_range() {
        let num_shards = 8;
        let assigner = ShardAssigner::new(num_shards);
        for _ in 0..1000 {
            let id = Uuid::new_v4();
            let shard = assigner.assign_shard(&id);
            assert!(shard < num_shards, "Shard {} >= num_shards {}", shard, num_shards);
        }
    }

    #[test]
    #[should_panic(expected = "num_shards must be > 0")]
    fn test_zero_shards_panics() {
        let _ = ShardAssigner::new(0);
    }

    #[test]
    fn test_distribution_roughly_uniform() {
        let num_shards = 4;
        let assigner = ShardAssigner::new(num_shards);
        let mut counts = vec![0u32; num_shards as usize];

        let n = 10_000;
        for _ in 0..n {
            let id = Uuid::new_v4();
            let shard = assigner.assign_shard(&id) as usize;
            counts[shard] += 1;
        }

        // Each shard should get roughly n/num_shards items.
        // Allow +/- 20% tolerance.
        let expected = n as f64 / num_shards as f64;
        for (i, count) in counts.iter().enumerate() {
            let ratio = *count as f64 / expected;
            assert!(
                ratio > 0.7 && ratio < 1.3,
                "Shard {} got {} items (expected ~{:.0}), ratio {:.2}",
                i,
                count,
                expected,
                ratio
            );
        }
    }
}
