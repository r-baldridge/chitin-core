// crates/chitin-daemon/src/state.rs
//
// Node state machine for the Chitin Protocol daemon.
//
// Valid transitions:
//   Initializing -> Syncing -> Ready -> Validating -> Ready
//   Any state -> ShuttingDown

use std::fmt;

/// Lifecycle states of the daemon node.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeState {
    /// Node is starting up and loading configuration.
    Initializing,
    /// Node is syncing state with the network.
    Syncing,
    /// Node is ready to accept requests and participate in consensus.
    Ready,
    /// Node is actively validating Polyps (Tide Node behavior).
    Validating,
    /// Node is shutting down gracefully.
    ShuttingDown,
}

impl fmt::Display for NodeState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NodeState::Initializing => write!(f, "Initializing"),
            NodeState::Syncing => write!(f, "Syncing"),
            NodeState::Ready => write!(f, "Ready"),
            NodeState::Validating => write!(f, "Validating"),
            NodeState::ShuttingDown => write!(f, "ShuttingDown"),
        }
    }
}

/// State machine for managing node lifecycle transitions.
pub struct NodeStateMachine {
    pub current: NodeState,
}

impl NodeStateMachine {
    /// Create a new state machine starting in the Initializing state.
    pub fn new() -> Self {
        Self {
            current: NodeState::Initializing,
        }
    }

    /// Attempt to transition to a new state.
    ///
    /// Returns an error if the transition is not valid.
    pub fn transition(&mut self, new_state: NodeState) -> Result<(), String> {
        // Any state can transition to ShuttingDown.
        if new_state == NodeState::ShuttingDown {
            tracing::info!(
                "State transition: {} -> {}",
                self.current,
                new_state
            );
            self.current = new_state;
            return Ok(());
        }

        let valid = match (&self.current, &new_state) {
            (NodeState::Initializing, NodeState::Syncing) => true,
            (NodeState::Syncing, NodeState::Ready) => true,
            (NodeState::Ready, NodeState::Validating) => true,
            (NodeState::Validating, NodeState::Ready) => true,
            _ => false,
        };

        if valid {
            tracing::info!(
                "State transition: {} -> {}",
                self.current,
                new_state
            );
            self.current = new_state;
            Ok(())
        } else {
            Err(format!(
                "Invalid state transition: {} -> {}",
                self.current, new_state
            ))
        }
    }
}

impl Default for NodeStateMachine {
    fn default() -> Self {
        Self::new()
    }
}
