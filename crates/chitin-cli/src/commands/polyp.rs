// crates/chitin-cli/src/commands/polyp.rs
//
// `chitin polyp {create, get, list}` — Polyp management commands.
//
// Phase 1: Print placeholder messages. Real RPC calls in Phase 2.

use clap::Subcommand;

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
        /// Filter by state: draft, soft, under_review, approved, hardened, rejected.
        #[arg(long)]
        state: Option<String>,
    },
}

/// Run the polyp subcommand.
pub async fn run(cmd: &PolypCmd) -> Result<(), Box<dyn std::error::Error>> {
    match cmd {
        PolypCmd::Create { text, content_type } => {
            println!("Creating Polyp...");
            println!("  Content type: {}", content_type);
            println!("  Text: {}", truncate(text, 80));
            println!();
            // Phase 1: placeholder. Real implementation will call RPC SubmitPolyp.
            let placeholder_id = uuid::Uuid::now_v7();
            println!("Polyp created (placeholder): {}", placeholder_id);
            println!("Note: Phase 1 placeholder. Real RPC submission in Phase 2.");
        }
        PolypCmd::Get { id } => {
            println!("Fetching Polyp: {}", id);
            println!();
            // Phase 1: placeholder.
            println!("Polyp not found (placeholder response).");
            println!("Note: Phase 1 placeholder. Real RPC lookup in Phase 2.");
        }
        PolypCmd::List { state } => {
            match state {
                Some(s) => println!("Listing Polyps with state: {}", s),
                None => println!("Listing all Polyps"),
            }
            println!();
            // Phase 1: placeholder.
            println!("No Polyps found (placeholder response).");
            println!("Note: Phase 1 placeholder. Real RPC listing in Phase 2.");
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
