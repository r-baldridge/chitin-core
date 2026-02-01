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

use chitin_core::identity::{NodeIdentity, NodeType};
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

    // ---------------------------------------------------------------
    // Phase 2: Load cryptographic identity from key files.
    // ---------------------------------------------------------------
    let (node_identity, signing_key) = load_node_identity(&daemon_config);

    if node_identity.is_placeholder() {
        tracing::warn!(
            "Running with placeholder identity — no key files found. \
             Generate keys with `chitin init`."
        );
    } else {
        tracing::info!("Node DID: {}", node_identity.did);
    }

    // Initialize the node state machine.
    let mut state_machine = NodeStateMachine::new();
    state_machine.transition(NodeState::Syncing)?;
    state_machine.transition(NodeState::Ready)?;

    // Start the appropriate node based on the configured type.
    match daemon_config.node_type.as_str() {
        "coral" => {
            let node = CoralNode::new(&daemon_config)?
                .with_identity(node_identity.clone(), signing_key);
            let store = node.store();
            let index = Arc::new(InMemoryVectorIndex::new());

            let rpc_config = RpcConfig {
                host: daemon_config.rpc_host.clone(),
                port: daemon_config.rpc_port,
            };
            let mut rpc_server = ChitinRpcServer::new(rpc_config, store.clone(), index.clone())
                .with_peer_info(daemon_config.peers.clone())
                .with_identity(node_identity.clone(), signing_key)
                .with_self_url(daemon_config.self_url.clone());

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

                // Set up gossip callback for polyp broadcast with real DID.
                let gossip_registry = registry.clone();
                let gossip_did = if !node_identity.is_placeholder() {
                    Some(node_identity.did.clone())
                } else {
                    None
                };
                rpc_server = rpc_server.with_gossip_callback(Arc::new(move |polyp| {
                    gossip::broadcast_polyp(gossip_registry.clone(), polyp, gossip_did.clone());
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
            let coral = CoralNode::new(&daemon_config)?
                .with_identity(node_identity.clone(), signing_key);
            let store = coral.store();
            let index = Arc::new(InMemoryVectorIndex::new());

            let rpc_config = RpcConfig {
                host: daemon_config.rpc_host.clone(),
                port: daemon_config.rpc_port,
            };
            let mut rpc_server = ChitinRpcServer::new(rpc_config, store.clone(), index.clone())
                .with_peer_info(daemon_config.peers.clone())
                .with_identity(node_identity.clone(), signing_key)
                .with_self_url(daemon_config.self_url.clone());

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

                // Set up gossip callback for polyp broadcast with real DID.
                let gossip_registry = registry.clone();
                let gossip_did = if !node_identity.is_placeholder() {
                    Some(node_identity.did.clone())
                } else {
                    None
                };
                rpc_server = rpc_server.with_gossip_callback(Arc::new(move |polyp| {
                    gossip::broadcast_polyp(gossip_registry.clone(), polyp, gossip_did.clone());
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

/// Load the node identity from key files on disk.
///
/// Reads the hotkey secret and coldkey public key from hex-encoded files,
/// derives the hotkey public key from the secret, and constructs a
/// `NodeIdentity`. Returns a placeholder identity if the files are not found.
fn load_node_identity(config: &DaemonConfig) -> (NodeIdentity, Option<[u8; 32]>) {
    let hotkey_path = expand_tilde(&config.hotkey_path);
    let coldkey_pub_path = expand_tilde(&config.coldkey_pub_path);

    let hotkey_secret = match std::fs::read_to_string(&hotkey_path) {
        Ok(hex_str) => match hex_decode(hex_str.trim()) {
            Some(bytes) if bytes.len() == 32 => {
                let mut arr = [0u8; 32];
                arr.copy_from_slice(&bytes);
                Some(arr)
            }
            _ => {
                tracing::warn!("Invalid hotkey secret at {}", hotkey_path);
                None
            }
        },
        Err(_) => {
            tracing::debug!("Hotkey secret not found at {}", hotkey_path);
            None
        }
    };

    let coldkey_pub = match std::fs::read_to_string(&coldkey_pub_path) {
        Ok(hex_str) => match hex_decode(hex_str.trim()) {
            Some(bytes) if bytes.len() == 32 => {
                let mut arr = [0u8; 32];
                arr.copy_from_slice(&bytes);
                Some(arr)
            }
            _ => {
                tracing::warn!("Invalid coldkey public key at {}", coldkey_pub_path);
                None
            }
        },
        Err(_) => {
            tracing::debug!("Coldkey public key not found at {}", coldkey_pub_path);
            None
        }
    };

    match (hotkey_secret, coldkey_pub) {
        (Some(secret), Some(coldkey)) => {
            // Derive hotkey public key from the secret.
            let signing_key = ed25519_dalek::SigningKey::from_bytes(&secret);
            let hotkey_pub = signing_key.verifying_key().to_bytes();

            // Determine node type from config.
            let node_type = match config.node_type.as_str() {
                "coral" => NodeType::Coral,
                "tide" => NodeType::Tide,
                _ => NodeType::Hybrid,
            };

            let identity = NodeIdentity::from_keypairs(hotkey_pub, coldkey, node_type);
            (identity, Some(secret))
        }
        _ => {
            // Placeholder identity when keys are not available.
            let identity = NodeIdentity {
                coldkey: [0u8; 32],
                hotkey: [0u8; 32],
                did: "did:chitin:local".to_string(),
                node_type: NodeType::Hybrid,
            };
            (identity, None)
        }
    }
}

/// Expand `~` at the start of a path to the user's home directory.
fn expand_tilde(path: &str) -> String {
    if path.starts_with("~/") {
        if let Some(home) = dirs::home_dir() {
            return format!("{}{}", home.display(), &path[1..]);
        }
    }
    path.to_string()
}

/// Decode a hex string into bytes. Returns None if the string is invalid hex.
fn hex_decode(hex: &str) -> Option<Vec<u8>> {
    if hex.len() % 2 != 0 {
        return None;
    }
    (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).ok())
        .collect()
}
