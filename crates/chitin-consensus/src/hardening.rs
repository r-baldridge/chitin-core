// crates/chitin-consensus/src/hardening.rs
//
// Hardening determination and CID anchoring for the Chitin Protocol.

use chitin_core::consensus::HardeningLineage;
use chitin_core::ChitinError;
use chitin_store::IpfsClient;
use chrono::Utc;
use sha2::{Digest, Sha256};
use uuid::Uuid;

/// Manages the hardening process for approved Polyps.
///
/// After Yuma-Semantic Consensus determines which Polyps are approved,
/// the HardeningManager finalizes them by pinning to IPFS, generating
/// Merkle proofs, and collecting attestations.
#[derive(Debug)]
pub struct HardeningManager {
    /// IPFS client for pinning hardened Polyps.
    pub ipfs: IpfsClient,
}

impl HardeningManager {
    /// Create a new HardeningManager with an IPFS client.
    pub fn new(ipfs: IpfsClient) -> Self {
        Self { ipfs }
    }

    /// Harden a Polyp by pinning to IPFS and generating a Merkle proof.
    ///
    /// # Arguments
    /// * `polyp_id` - The UUID of the Polyp to harden.
    /// * `cid` - The IPFS CID of the serialized Polyp.
    ///
    /// Returns a `HardeningLineage` with the CID, Merkle root, and timestamp.
    pub async fn harden_polyp(
        &self,
        polyp_id: Uuid,
        cid: String,
    ) -> Result<HardeningLineage, ChitinError> {
        // 1. Pin CID to IPFS
        self.ipfs.pin(&cid).await?;

        // 2. Compute Merkle leaf: SHA-256(polyp_id_bytes || cid_bytes)
        let mut hasher = Sha256::new();
        hasher.update(polyp_id.as_bytes());
        hasher.update(cid.as_bytes());
        let merkle_leaf: [u8; 32] = hasher.finalize().into();

        // 3. Single-leaf Merkle tree: root = leaf, proof = empty
        let merkle_root = merkle_leaf;

        // 4. Return HardeningLineage
        Ok(HardeningLineage {
            cid,
            merkle_proof: vec![],
            merkle_root,
            attestations: vec![],
            anchor_tx: None,
            hardened_at: Utc::now(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    async fn mock_ipfs_pin_server() -> (String, tokio::task::JoinHandle<()>) {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let base_url = format!("http://{}", addr);
        let body = r#"{"Pins":["QmTest"]}"#;
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{}",
            body.len(),
            body
        );

        let handle = tokio::spawn(async move {
            if let Ok((mut stream, _)) = listener.accept().await {
                let mut buf = vec![0u8; 4096];
                let _ = stream.read(&mut buf).await;
                let _ = stream.write_all(response.as_bytes()).await;
            }
        });

        (base_url, handle)
    }

    #[tokio::test]
    async fn hardening_generates_valid_merkle_root() {
        let (base_url, _handle) = mock_ipfs_pin_server().await;
        let manager = HardeningManager::new(IpfsClient::new(&base_url));
        let polyp_id = Uuid::now_v7();
        let cid = "QmTestCid123".to_string();

        let lineage = manager.harden_polyp(polyp_id, cid.clone()).await.unwrap();

        // Verify Merkle root matches expected hash
        let mut hasher = Sha256::new();
        hasher.update(polyp_id.as_bytes());
        hasher.update(cid.as_bytes());
        let expected_root: [u8; 32] = hasher.finalize().into();

        assert_eq!(lineage.merkle_root, expected_root);
        assert_eq!(lineage.cid, "QmTestCid123");
        assert!(lineage.merkle_proof.is_empty());
        assert!(lineage.attestations.is_empty());
        assert!(lineage.anchor_tx.is_none());
    }

    #[tokio::test]
    async fn hardening_returns_populated_lineage() {
        let (base_url, _handle) = mock_ipfs_pin_server().await;
        let manager = HardeningManager::new(IpfsClient::new(&base_url));
        let polyp_id = Uuid::now_v7();

        let lineage = manager
            .harden_polyp(polyp_id, "QmABC".to_string())
            .await
            .unwrap();

        assert_eq!(lineage.cid, "QmABC");
        assert!(!lineage.merkle_root.iter().all(|&b| b == 0)); // Non-zero root
    }
}
