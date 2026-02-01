// crates/chitin-cli/src/commands/query.rs
//
// `chitin query <text>` — semantic search against the Reef.
//
// Phase 1: Print placeholder search message. Real RPC calls in Phase 2.

use clap::Args;

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
pub async fn run(cmd: &QueryCmd) -> Result<(), Box<dyn std::error::Error>> {
    println!("Searching for: \"{}\"", cmd.text);
    println!("  Model:  {}", cmd.model);
    println!("  Top-K:  {}", cmd.top_k);
    println!();
    // Phase 1: placeholder.
    println!("No results found (placeholder response).");
    println!("Note: Phase 1 placeholder. Real semantic search in Phase 2.");

    Ok(())
}
