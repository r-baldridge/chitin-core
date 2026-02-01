// crates/chitin-store/src/lib.rs
//
// chitin-store: Storage layer for the Chitin Protocol.
//
// Provides RocksDB-backed Polyp persistence, IPFS client stubs for
// content-addressed immutable storage, a hardened store for CID-indexed
// Polyps, an in-memory vector index (Phase 1 placeholder for Qdrant),
// Bloom filters for set membership, and consistent-hash shard assignment.

pub mod bloom;
pub mod hardened;
pub mod hnsw;
pub mod ipfs;
pub mod rocks;
pub mod shard;

// Re-export key types for ergonomic access from downstream crates.
pub use bloom::PolypBloomFilter;
pub use hardened::HardenedStore;
pub use hnsw::InMemoryVectorIndex;
pub use ipfs::IpfsClient;
pub use rocks::RocksStore;
pub use shard::ShardAssigner;
