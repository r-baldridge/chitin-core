// crates/chitin-daemon/src/config.rs
//
// Runtime configuration for the Chitin Protocol daemon.
// Loaded from a TOML file or populated with sensible defaults.

use serde::Deserialize;
use std::fs;

/// Runtime configuration for the daemon.
#[derive(Debug, Clone, Deserialize)]
pub struct DaemonConfig {
    /// Node type: "coral", "tide", or "hybrid".
    #[serde(default = "default_node_type")]
    pub node_type: String,

    /// Directory for local data storage (RocksDB, keys, etc.).
    #[serde(default = "default_data_dir")]
    pub data_dir: String,

    /// Host address for the RPC server.
    #[serde(default = "default_rpc_host")]
    pub rpc_host: String,

    /// Port for the RPC server.
    #[serde(default = "default_rpc_port")]
    pub rpc_port: u16,

    /// Port for P2P communication.
    #[serde(default = "default_p2p_port")]
    pub p2p_port: u16,

    /// URL of the IPFS API endpoint.
    #[serde(default = "default_ipfs_api_url")]
    pub ipfs_api_url: String,

    /// Log level: "trace", "debug", "info", "warn", "error".
    #[serde(default = "default_log_level")]
    pub log_level: String,

    /// Peer URLs for HTTP relay (e.g., ["http://10.0.0.2:50051"]).
    /// When empty (default), all peer networking is disabled.
    #[serde(default)]
    pub peers: Vec<String>,

    /// This node's publicly reachable URL (e.g., "http://10.0.0.1:50051").
    /// Used in peer announcements so other nodes know how to reach us.
    #[serde(default)]
    pub self_url: Option<String>,
}

fn default_node_type() -> String {
    "hybrid".to_string()
}

fn default_data_dir() -> String {
    "~/.chitin/data".to_string()
}

fn default_rpc_host() -> String {
    "127.0.0.1".to_string()
}

fn default_rpc_port() -> u16 {
    50051
}

fn default_p2p_port() -> u16 {
    4001
}

fn default_ipfs_api_url() -> String {
    "http://127.0.0.1:5001".to_string()
}

fn default_log_level() -> String {
    "info".to_string()
}

impl Default for DaemonConfig {
    fn default() -> Self {
        Self {
            node_type: default_node_type(),
            data_dir: default_data_dir(),
            rpc_host: default_rpc_host(),
            rpc_port: default_rpc_port(),
            p2p_port: default_p2p_port(),
            ipfs_api_url: default_ipfs_api_url(),
            log_level: default_log_level(),
            peers: Vec::new(),
            self_url: None,
        }
    }
}

impl DaemonConfig {
    /// Load configuration from a TOML file at the given path.
    ///
    /// Returns an error if the file cannot be read or parsed.
    pub fn load(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let contents = fs::read_to_string(path)?;
        let config: DaemonConfig = toml::from_str(&contents)?;
        Ok(config)
    }
}
