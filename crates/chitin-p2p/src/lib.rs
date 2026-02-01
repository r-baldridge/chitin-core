// crates/chitin-p2p/src/lib.rs
//
// chitin-p2p: libp2p networking layer for the Chitin Protocol.

pub mod transport;
pub mod discovery;
pub mod gossip;
pub mod axon;
pub mod dendrite;
pub mod behaviour;

use std::sync::Arc;
use tokio::sync::Mutex;
use libp2p::Swarm;
use crate::behaviour::ChitinBehaviour;

/// Shared handle to the libp2p Swarm, safe for concurrent access.
pub type SwarmHandle = Arc<Mutex<Swarm<ChitinBehaviour>>>;
