// crates/chitin-sync/src/lib.rs
//
// chitin-sync: Vector Bloom Filters, set reconciliation, and range sync
// for the Chitin Protocol.
//
// This crate enables efficient synchronization of Polyp sets between nodes.
// Vector Bloom Filters provide compact set summaries, set reconciliation
// identifies missing Polyps, and range sync handles shard catchup.

pub mod vbf;
pub mod reconcile;
pub mod range;
