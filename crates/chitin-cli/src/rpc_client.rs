// crates/chitin-cli/src/rpc_client.rs
//
// Lightweight JSON-RPC client that POSTs to the chitin-daemon HTTP endpoint.

use serde::{Deserialize, Serialize};

/// Mirrors the server's JsonRpcRequest envelope.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub method: String,
    pub params: serde_json::Value,
}

/// Mirrors the server's JsonRpcResponse envelope.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    pub success: bool,
    pub result: Option<serde_json::Value>,
    pub error: Option<String>,
}

/// Send a JSON-RPC call to the daemon and return the parsed response.
pub async fn rpc_call(
    endpoint: &str,
    method: &str,
    params: serde_json::Value,
) -> Result<JsonRpcResponse, Box<dyn std::error::Error>> {
    let request = JsonRpcRequest {
        method: method.to_string(),
        params,
    };

    let client = reqwest::Client::new();
    let resp = client
        .post(endpoint)
        .json(&request)
        .send()
        .await?;

    let rpc_response: JsonRpcResponse = resp.json().await?;
    Ok(rpc_response)
}
