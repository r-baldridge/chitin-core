// crates/chitin-consensus/src/lib.rs
//
// chitin-consensus: Yuma-Semantic consensus, scoring, weights, bonds, and epochs
// for the Chitin Protocol.
//
// This crate implements the core consensus algorithm that determines which Polyps
// are hardened into the Reef, and how rewards are distributed among Coral and Tide Nodes.

pub mod yuma;
pub mod scoring;
pub mod weights;
pub mod bonds;
pub mod epoch;
pub mod metagraph;
pub mod hardening;
