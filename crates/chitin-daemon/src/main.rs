// crates/chitin-daemon/src/main.rs
//
// Binary entrypoint for the Chitin Protocol daemon.
//
// Initializes tracing, parses CLI arguments, loads configuration,
// and starts the appropriate node type (Coral, Tide, or Hybrid).

mod config;
mod coral;
mod gossip;
mod peers;
mod scheduler;
mod state;
mod sync_loop;
mod tide;

use std::sync::Arc;

use clap::Parser;
use config::DaemonConfig;
use coral::CoralNode;
use state::{NodeState, NodeStateMachine};
use tide::TideNode;

use chitin_rpc::{ChitinRpcServer, RpcConfig};
use chitin_store::InMemoryVectorIndex;
use peers::PeerRegistry;

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
            let store = node.store();
            let index = Arc::new(InMemoryVectorIndex::new());

            let rpc_config = RpcConfig {
                host: daemon_config.rpc_host.clone(),
                port: daemon_config.rpc_port,
            };
            let mut rpc_server = ChitinRpcServer::new(rpc_config, store.clone(), index.clone())
                .with_peer_info(daemon_config.peers.clone());

            // Wire up peer networking if peers are configured.
            if !daemon_config.peers.is_empty() {
                let registry = Arc::new(PeerRegistry::new(
                    daemon_config.self_url.clone(),
                    daemon_config.peers.clone(),
                ));
                tracing::info!(
                    "Peer networking enabled: {} peers configured",
                    daemon_config.peers.len()
                );

                // Set up gossip callback for polyp broadcast.
                let gossip_registry = registry.clone();
                rpc_server = rpc_server.with_gossip_callback(Arc::new(move |polyp| {
                    gossip::broadcast_polyp(gossip_registry.clone(), polyp, None);
                }));

                // Spawn announce to all peers.
                let announce_registry = registry.clone();
                tokio::spawn(async move {
                    announce_registry.announce_to_all().await;
                });

                // Spawn sync loop (30s interval).
                let sync_registry = registry.clone();
                let sync_store = store.clone();
                let sync_index = index.clone();
                tokio::spawn(async move {
                    sync_loop::run_sync_loop(sync_registry, sync_store, sync_index, 30).await;
                });
            }

            // Spawn RPC server in background, run node in foreground.
            tokio::spawn(async move {
                if let Err(e) = rpc_server.start().await {
                    tracing::error!("RPC server error: {}", e);
                }
            });

            node.start().await?;
        }
        "tide" => {
            let node = TideNode::new(&daemon_config)?;
            node.start().await?;
        }
        "hybrid" => {
            tracing::info!("Running in Hybrid mode (Coral + Tide)");
            let coral = CoralNode::new(&daemon_config)?;
            let store = coral.store();
            let index = Arc::new(InMemoryVectorIndex::new());

            let rpc_config = RpcConfig {
                host: daemon_config.rpc_host.clone(),
                port: daemon_config.rpc_port,
            };
            let mut rpc_server = ChitinRpcServer::new(rpc_config, store.clone(), index.clone())
                .with_peer_info(daemon_config.peers.clone());

            // Wire up peer networking if peers are configured.
            if !daemon_config.peers.is_empty() {
                let registry = Arc::new(PeerRegistry::new(
                    daemon_config.self_url.clone(),
                    daemon_config.peers.clone(),
                ));
                tracing::info!(
                    "Peer networking enabled: {} peers configured",
                    daemon_config.peers.len()
                );

                // Set up gossip callback for polyp broadcast.
                let gossip_registry = registry.clone();
                rpc_server = rpc_server.with_gossip_callback(Arc::new(move |polyp| {
                    gossip::broadcast_polyp(gossip_registry.clone(), polyp, None);
                }));

                // Spawn announce to all peers.
                let announce_registry = registry.clone();
                tokio::spawn(async move {
                    announce_registry.announce_to_all().await;
                });

                // Spawn sync loop (30s interval).
                let sync_registry = registry.clone();
                let sync_store = store.clone();
                let sync_index = index.clone();
                tokio::spawn(async move {
                    sync_loop::run_sync_loop(sync_registry, sync_store, sync_index, 30).await;
                });
            }

            let tide = TideNode::new(&daemon_config)?;

            // Spawn RPC server in background, run both nodes concurrently.
            tokio::spawn(async move {
                if let Err(e) = rpc_server.start().await {
                    tracing::error!("RPC server error: {}", e);
                }
            });

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
