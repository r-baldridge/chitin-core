// crates/chitin-reputation/src/lib.rs
//
// chitin-reputation: Trust matrix, OpenRank, domain context, and decay
// for the Chitin Protocol.
//
// This crate manages node reputation and trust scoring. Trust is domain-scoped
// (a node can be highly trusted in "medical" but not in "code") and decays
// over time to ensure ongoing participation.

pub mod trust_matrix;
pub mod openrank;
pub mod domain;
pub mod decay;
