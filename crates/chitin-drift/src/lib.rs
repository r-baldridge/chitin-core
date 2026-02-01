// crates/chitin-drift/src/lib.rs
//
// chitin-drift: Drift detection, molting, alignment, and versioning
// for the Chitin Protocol.
//
// This crate handles the lifecycle of embedding models: detecting when
// a new model causes semantic drift, orchestrating the "molting" process
// to re-embed Polyps, computing alignment matrices between model spaces,
// and managing model version registries.

pub mod detection;
pub mod molting;
pub mod alignment;
pub mod versioning;
