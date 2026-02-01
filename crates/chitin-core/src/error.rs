use thiserror::Error;

/// Protocol-wide error types for the Chitin Protocol.
#[derive(Debug, Error)]
pub enum ChitinError {
    /// Storage layer error (RocksDB, IPFS, vector index).
    #[error("Storage error: {0}")]
    Storage(String),

    /// Verification error (ZK proof verification failed).
    #[error("Verification error: {0}")]
    Verification(String),

    /// Consensus error (scoring, weight submission, epoch management).
    #[error("Consensus error: {0}")]
    Consensus(String),

    /// Network error (P2P transport, gossip, discovery).
    #[error("Network error: {0}")]
    Network(String),

    /// Cryptographic error (key generation, signing, verification).
    #[error("Crypto error: {0}")]
    Crypto(String),

    /// Serialization/deserialization error.
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Invalid state transition.
    #[error("Invalid state: {0}")]
    InvalidState(String),

    /// Resource not found.
    #[error("Not found: {0}")]
    NotFound(String),
}

impl From<serde_json::Error> for ChitinError {
    fn from(e: serde_json::Error) -> Self {
        ChitinError::Serialization(e.to_string())
    }
}

impl From<ed25519_dalek::SignatureError> for ChitinError {
    fn from(e: ed25519_dalek::SignatureError) -> Self {
        ChitinError::Crypto(e.to_string())
    }
}
