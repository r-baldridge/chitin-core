// crates/chitin-daemon/src/epoch_events.rs
//
// Epoch event types broadcast from the scheduler to daemon tasks.
//
// The EpochScheduler publishes events on a tokio broadcast channel.
// TideNode and the consensus runner subscribe to receive phase transitions.

use chitin_consensus::epoch::EpochPhase;

/// Events emitted by the epoch scheduler during block progression.
#[derive(Debug, Clone)]
pub enum EpochEvent {
    /// An epoch phase transition occurred.
    PhaseChanged {
        /// Current epoch number.
        epoch: u64,
        /// The new phase.
        phase: EpochPhase,
        /// Block height at which the transition occurred.
        block: u64,
    },
    /// The epoch boundary was crossed — a new epoch has begun.
    EpochBoundary {
        /// The new epoch number (just started).
        epoch: u64,
        /// Block height at the boundary.
        block: u64,
    },
}
