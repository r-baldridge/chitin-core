// crates/chitin-cli/src/commands/polyp.rs
//
// `chitin polyp {create, get, list}` — Polyp management commands.

use clap::Subcommand;

use crate::rpc_client::rpc_call;

/// Polyp management subcommands.
#[derive(Debug, Subcommand)]
pub enum PolypCmd {
    /// Create a new Polyp from text input.
    Create {
        /// The text content for the Polyp.
        #[arg(long)]
        text: String,
        /// MIME type of the content (default: text/plain).
        #[arg(long, default_value = "text/plain")]
        content_type: String,
    },
    /// Get a Polyp by its UUID.
    Get {
        /// The UUID of the Polyp to retrieve.
        #[arg(long)]
        id: String,
    },
    /// List Polyps, optionally filtered by lifecycle state.
    List {
        /// Filter by state: Draft, Soft, UnderReview, Approved, Hardened, Rejected.
        #[arg(long)]
        state: Option<String>,
    },
}

/// Run the polyp subcommand.
pub async fn run(cmd: &PolypCmd, rpc_endpoint: &str) -> Result<(), Box<dyn std::error::Error>> {
    match cmd {
        PolypCmd::Create { text, content_type } => {
            let params = serde_json::json!({
                "content": text,
                "content_type": content_type,
                "language": "en",
            });

            let resp = rpc_call(rpc_endpoint, "polyp/submit", params).await?;

            if resp.success {
                if let Some(result) = &resp.result {
                    let polyp_id = result
                        .get("polyp_id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown");
                    let state = result
                        .get("state")
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown");
                    println!("Polyp created successfully");
                    println!("  ID:    {}", polyp_id);
                    println!("  State: {}", state);
                }
            } else {
                eprintln!(
                    "Error: {}",
                    resp.error.unwrap_or_else(|| "Unknown error".to_string())
                );
            }
        }
        PolypCmd::Get { id } => {
            let params = serde_json::json!({
                "polyp_id": id,
            });

            let resp = rpc_call(rpc_endpoint, "polyp/get", params).await?;

            if resp.success {
                if let Some(result) = &resp.result {
                    let found = result.get("found").and_then(|v| v.as_bool()).unwrap_or(false);
                    if found {
                        if let Some(polyp) = result.get("polyp") {
                            println!("{}", serde_json::to_string_pretty(polyp)?);
                        }
                    } else {
                        println!("Polyp not found: {}", id);
                    }
                }
            } else {
                eprintln!(
                    "Error: {}",
                    resp.error.unwrap_or_else(|| "Unknown error".to_string())
                );
            }
        }
        PolypCmd::List { state } => {
            let params = serde_json::json!({
                "state_filter": state,
                "limit": 100,
                "offset": 0,
            });

            let resp = rpc_call(rpc_endpoint, "polyp/list", params).await?;

            if resp.success {
                if let Some(result) = &resp.result {
                    let total = result.get("total").and_then(|v| v.as_u64()).unwrap_or(0);
                    let polyps = result
                        .get("polyps")
                        .and_then(|v| v.as_array())
                        .cloned()
                        .unwrap_or_default();

                    println!("Polyps ({} total):", total);
                    println!("{:<38} {:<10} {}", "ID", "State", "Content");
                    println!("{}", "-".repeat(80));
                    for p in &polyps {
                        let id = p.get("id").and_then(|v| v.as_str()).unwrap_or("?");
                        let st = p
                            .get("state")
                            .map(|v| format!("{}", v))
                            .unwrap_or_else(|| "?".to_string());
                        let content = p
                            .get("subject")
                            .and_then(|s| s.get("payload"))
                            .and_then(|p| p.get("content"))
                            .and_then(|c| c.as_str())
                            .unwrap_or("");
                        println!("{:<38} {:<10} {}", id, st, truncate(content, 40));
                    }
                }
            } else {
                eprintln!(
                    "Error: {}",
                    resp.error.unwrap_or_else(|| "Unknown error".to_string())
                );
            }
        }
    }

    Ok(())
}

/// Truncate a string to the given maximum length, appending "..." if truncated.
fn truncate(s: &str, max_len: usize) -> String {
    if s.len() > max_len {
        format!("{}...", &s[..max_len])
    } else {
        s.to_string()
    }
}
