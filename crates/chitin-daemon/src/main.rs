// crates/chitin-daemon/src/main.rs
//
// Binary entrypoint for the Chitin Protocol daemon.
//
// Initializes tracing, parses CLI arguments, loads configuration,
// and starts the appropriate node type (Coral, Tide, or Hybrid).

mod config;
mod coral;
mod scheduler;
mod state;
mod tide;

use clap::Parser;
use config::DaemonConfig;
use coral::CoralNode;
use state::{NodeState, NodeStateMachine};
use tide::TideNode;

/// Chitin Protocol daemon — runs Coral and/or Tide node processes.
#[derive(Parser, Debug)]
#[command(name = "chitin-daemon", version = "0.1.0", about = "Chitin Protocol node daemon")]
struct Args {
    /// Path to the TOML configuration file.
    #[arg(long, default_value = "~/.chitin/config.toml")]
    config: String,

    /// Node type to run: coral, tide, or hybrid.
    #[arg(long, default_value = "hybrid")]
    node_type: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing subscriber for structured logging.
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let args = Args::parse();

    // Load configuration from TOML file, falling back to defaults if the file
    // is not found.
    let mut daemon_config = match DaemonConfig::load(&args.config) {
        Ok(cfg) => {
            tracing::info!("Loaded configuration from {}", args.config);
            cfg
        }
        Err(e) => {
            tracing::warn!(
                "Could not load config from {}: {}. Using defaults.",
                args.config,
                e
            );
            DaemonConfig::default()
        }
    };

    // CLI --node-type flag overrides the config file value.
    daemon_config.node_type = args.node_type.clone();

    tracing::info!("Chitin Protocol Daemon v0.1.0");
    tracing::info!("Node type: {}", daemon_config.node_type);
    tracing::info!("Data directory: {}", daemon_config.data_dir);
    tracing::info!(
        "RPC endpoint: {}:{}",
        daemon_config.rpc_host,
        daemon_config.rpc_port
    );
    tracing::info!("P2P port: {}", daemon_config.p2p_port);

    // Initialize the node state machine.
    let mut state_machine = NodeStateMachine::new();
    state_machine.transition(NodeState::Syncing)?;
    state_machine.transition(NodeState::Ready)?;

    // Start the appropriate node based on the configured type.
    match daemon_config.node_type.as_str() {
        "coral" => {
            let node = CoralNode::new(&daemon_config)?;
            node.start().await?;
        }
        "tide" => {
            let node = TideNode::new(&daemon_config)?;
            node.start().await?;
        }
        "hybrid" => {
            tracing::info!("Running in Hybrid mode (Coral + Tide)");
            let coral = CoralNode::new(&daemon_config)?;
            let tide = TideNode::new(&daemon_config)?;

            // Run both nodes concurrently. First to complete (shutdown) wins.
            tokio::select! {
                result = coral.start() => {
                    if let Err(e) = result {
                        tracing::error!("Coral node error: {}", e);
                    }
                }
                result = tide.start() => {
                    if let Err(e) = result {
                        tracing::error!("Tide node error: {}", e);
                    }
                }
            }
        }
        other => {
            tracing::error!("Unknown node type: {}. Use 'coral', 'tide', or 'hybrid'.", other);
            return Err(format!("Unknown node type: {}", other).into());
        }
    }

    // Transition to shutting down.
    let _ = state_machine.transition(NodeState::ShuttingDown);
    tracing::info!("Chitin daemon shut down gracefully");

    Ok(())
}
