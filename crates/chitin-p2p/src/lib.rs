// crates/chitin-p2p/src/lib.rs
//
// chitin-p2p: libp2p networking layer for the Chitin Protocol.
//
// Provides transport, peer discovery, gossip-based Polyp broadcast,
// and axon/dendrite message passing between Coral and Tide Nodes.

pub mod transport;
pub mod discovery;
pub mod gossip;
pub mod axon;
pub mod dendrite;
pub mod behaviour;
