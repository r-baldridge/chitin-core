// crates/chitin-cli/src/commands/status.rs
//
// `chitin status` — display node connection status and version info.

use crate::rpc_client::rpc_call;

/// Run the status command.
pub async fn run(rpc_endpoint: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("Chitin Protocol v0.1.0");
    println!();

    let resp = rpc_call(rpc_endpoint, "node/health", serde_json::json!({})).await;

    match resp {
        Ok(r) if r.success => {
            println!("Node Status");
            println!("-----------");
            println!("  Connection:   CONNECTED");
            println!("  RPC endpoint: {}", rpc_endpoint);

            if let Some(result) = &r.result {
                let status = result
                    .get("status")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");
                let storage_ok = result
                    .get("storage_ok")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                let index_ok = result
                    .get("index_ok")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                println!("  Health:       {}", status);
                println!("  Storage:      {}", if storage_ok { "OK" } else { "DEGRADED" });
                println!("  Index:        {}", if index_ok { "OK" } else { "DEGRADED" });
            }
        }
        Ok(r) => {
            println!("Node Status");
            println!("-----------");
            println!("  Connection:   CONNECTED (with errors)");
            println!("  RPC endpoint: {}", rpc_endpoint);
            if let Some(err) = &r.error {
                println!("  Error:        {}", err);
            }
        }
        Err(_) => {
            println!("Node Status");
            println!("-----------");
            println!("  Connection:   NOT CONNECTED");
            println!("  RPC endpoint: {}", rpc_endpoint);
            println!();
            println!("Could not reach daemon. Is chitin-daemon running?");
        }
    }

    Ok(())
}
