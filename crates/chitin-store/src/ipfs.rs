// crates/chitin-store/src/ipfs.rs
//
// IPFS client for content-addressed immutable storage.
// Uses reqwest to communicate with a Kubo/IPFS daemon HTTP API.

use chitin_core::ChitinError;

/// IPFS client for interacting with a Kubo / IPFS daemon.
///
/// Communicates with the IPFS HTTP API using reqwest.
#[derive(Debug, Clone)]
pub struct IpfsClient {
    /// Base URL of the IPFS HTTP API (e.g., "http://127.0.0.1:5001").
    pub base_url: String,
    /// HTTP client instance.
    client: reqwest::Client,
}

impl IpfsClient {
    /// Create a new IPFS client pointing at the given API base URL.
    pub fn new(base_url: &str) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            client: reqwest::Client::new(),
        }
    }

    /// Pin a CID to the local IPFS node, ensuring the data is retained.
    ///
    /// POST /api/v0/pin/add?arg={cid}
    pub async fn pin(&self, cid: &str) -> Result<(), ChitinError> {
        let url = format!("{}/api/v0/pin/add?arg={}", self.base_url, cid);
        let response = self
            .client
            .post(&url)
            .send()
            .await
            .map_err(|e| ChitinError::Storage(format!("IPFS pin request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(ChitinError::Storage(format!(
                "IPFS pin failed ({}): {}",
                status, body
            )));
        }

        Ok(())
    }

    /// Unpin a CID from the local IPFS node, allowing garbage collection.
    ///
    /// POST /api/v0/pin/rm?arg={cid}
    pub async fn unpin(&self, cid: &str) -> Result<(), ChitinError> {
        let url = format!("{}/api/v0/pin/rm?arg={}", self.base_url, cid);
        let response = self
            .client
            .post(&url)
            .send()
            .await
            .map_err(|e| ChitinError::Storage(format!("IPFS unpin request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(ChitinError::Storage(format!(
                "IPFS unpin failed ({}): {}",
                status, body
            )));
        }

        Ok(())
    }

    /// Retrieve raw bytes for a given CID from the IPFS network.
    ///
    /// POST /api/v0/cat?arg={cid}
    pub async fn get_by_cid(&self, cid: &str) -> Result<Vec<u8>, ChitinError> {
        let url = format!("{}/api/v0/cat?arg={}", self.base_url, cid);
        let response = self
            .client
            .post(&url)
            .send()
            .await
            .map_err(|e| ChitinError::Storage(format!("IPFS get request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(ChitinError::Storage(format!(
                "IPFS get failed ({}): {}",
                status, body
            )));
        }

        let bytes = response
            .bytes()
            .await
            .map_err(|e| ChitinError::Storage(format!("IPFS get body read failed: {}", e)))?;

        Ok(bytes.to_vec())
    }

    /// Store raw bytes on IPFS and return the resulting CID.
    ///
    /// POST /api/v0/add with multipart form data.
    pub async fn put(&self, data: &[u8]) -> Result<String, ChitinError> {
        let url = format!("{}/api/v0/add", self.base_url);

        let part = reqwest::multipart::Part::bytes(data.to_vec()).file_name("data");
        let form = reqwest::multipart::Form::new().part("file", part);

        let response = self
            .client
            .post(&url)
            .multipart(form)
            .send()
            .await
            .map_err(|e| ChitinError::Storage(format!("IPFS put request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(ChitinError::Storage(format!(
                "IPFS put failed ({}): {}",
                status, body
            )));
        }

        // Parse JSON response to extract CID from the "Hash" field
        let body: serde_json::Value = response
            .json()
            .await
            .map_err(|e| ChitinError::Serialization(format!("IPFS add response parse failed: {}", e)))?;

        let cid = body["Hash"]
            .as_str()
            .ok_or_else(|| {
                ChitinError::Serialization("IPFS add response missing 'Hash' field".to_string())
            })?
            .to_string();

        Ok(cid)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    /// Helper to start a mock IPFS HTTP server that returns a fixed response.
    async fn mock_ipfs_server(response_body: &str) -> (String, tokio::task::JoinHandle<()>) {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let base_url = format!("http://{}", addr);
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
            response_body.len(),
            response_body
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

    /// Helper for a mock server that returns an error status.
    async fn mock_ipfs_error_server(status: u16) -> (String, tokio::task::JoinHandle<()>) {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let base_url = format!("http://{}", addr);
        let body = r#"{"Message":"error","Code":0}"#;
        let response = format!(
            "HTTP/1.1 {} Error\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
            status,
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
    async fn put_returns_cid() {
        let (base_url, _handle) =
            mock_ipfs_server(r#"{"Hash":"QmTest123","Size":"11"}"#).await;
        let client = IpfsClient::new(&base_url);
        let result = client.put(b"hello world").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "QmTest123");
    }

    #[tokio::test]
    async fn get_by_cid_returns_data() {
        let (base_url, _handle) = mock_ipfs_server("hello world").await;
        let client = IpfsClient::new(&base_url);
        let result = client.get_by_cid("QmTest123").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), b"hello world");
    }

    #[tokio::test]
    async fn pin_succeeds() {
        let (base_url, _handle) =
            mock_ipfs_server(r#"{"Pins":["QmTest123"]}"#).await;
        let client = IpfsClient::new(&base_url);
        let result = client.pin("QmTest123").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn unpin_succeeds() {
        let (base_url, _handle) =
            mock_ipfs_server(r#"{"Pins":["QmTest123"]}"#).await;
        let client = IpfsClient::new(&base_url);
        let result = client.unpin("QmTest123").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn connection_error_returns_chitin_error() {
        let client = IpfsClient::new("http://127.0.0.1:1"); // Nothing listening
        let result = client.put(b"test").await;
        assert!(result.is_err());
        match result.unwrap_err() {
            ChitinError::Storage(msg) => assert!(msg.contains("request failed")),
            other => panic!("Expected Storage error, got: {:?}", other),
        }
    }
}
