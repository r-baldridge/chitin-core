// crates/chitin-cli/src/commands/query.rs
//
// `chitin query <text>` — semantic search against the Reef.

use clap::Args;

use crate::rpc_client::rpc_call;

/// Semantic search query command.
#[derive(Debug, Args)]
pub struct QueryCmd {
    /// The natural language query text.
    #[arg()]
    pub text: String,

    /// Number of results to return (default: 10).
    #[arg(long, default_value = "10")]
    pub top_k: usize,

    /// Embedding model to use for the query vector.
    #[arg(long, default_value = "bge/bge-small-en-v1.5")]
    pub model: String,
}

/// Run the query command.
pub async fn run(cmd: &QueryCmd, rpc_endpoint: &str) -> Result<(), Box<dyn std::error::Error>> {
    let params = serde_json::json!({
        "query_text": cmd.text,
        "top_k": cmd.top_k,
        "model_id": cmd.model,
    });

    let resp = rpc_call(rpc_endpoint, "query/search", params).await?;

    if resp.success {
        if let Some(result) = &resp.result {
            let results = result
                .get("results")
                .and_then(|v| v.as_array())
                .cloned()
                .unwrap_or_default();
            let search_time = result
                .get("search_time_ms")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let total = result
                .get("total_found")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);

            println!(
                "Search results: {} found ({} ms)",
                total, search_time
            );
            println!();

            if results.is_empty() {
                println!("No results found.");
            } else {
                println!(
                    "{:<38} {:<10} {:<10} {}",
                    "Polyp ID", "Sim", "State", "Content"
                );
                println!("{}", "-".repeat(90));
                for r in &results {
                    let id = r.get("polyp_id").and_then(|v| v.as_str()).unwrap_or("?");
                    let sim = r
                        .get("similarity")
                        .and_then(|v| v.as_f64())
                        .unwrap_or(0.0);
                    let state = r.get("state").and_then(|v| v.as_str()).unwrap_or("?");
                    let content = r.get("content").and_then(|v| v.as_str()).unwrap_or("");
                    let truncated = if content.len() > 40 {
                        format!("{}...", &content[..40])
                    } else {
                        content.to_string()
                    };
                    println!("{:<38} {:<10.4} {:<10} {}", id, sim, state, truncated);
                }
            }
        }
    } else {
        eprintln!(
            "Error: {}",
            resp.error.unwrap_or_else(|| "Unknown error".to_string())
        );
    }

    Ok(())
}
