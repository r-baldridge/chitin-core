// crates/chitin-cli/src/main.rs
//
// CLI entrypoint for the Chitin Protocol developer tools.
//
// Provides subcommands for initializing a node, managing wallets,
// creating and querying Polyps, staking, and viewing network status.

mod commands;
mod output;

use clap::{Parser, Subcommand};
use commands::polyp::PolypCmd;
use commands::query::QueryCmd;
use commands::stake::StakeCmd;
use commands::wallet::WalletCmd;

/// Chitin Protocol CLI — developer tools for Reefipedia.
#[derive(Parser, Debug)]
#[command(
    name = "chitin",
    version = "0.1.0",
    about = "Chitin Protocol CLI for Reefipedia — decentralized semantic knowledge store"
)]
struct Cli {
    /// RPC endpoint for the chitin-daemon.
    #[arg(long, global = true, default_value = "http://localhost:50051")]
    rpc: String,

    #[command(subcommand)]
    command: Commands,
}

/// Top-level subcommands.
#[derive(Debug, Subcommand)]
enum Commands {
    /// Initialize Chitin configuration and generate keypair.
    Init,

    /// Wallet management: create, import, export keys.
    #[command(subcommand)]
    Wallet(WalletCmd),

    /// Polyp management: create, get, list.
    #[command(subcommand)]
    Polyp(PolypCmd),

    /// Semantic search against the Reef.
    Query(QueryCmd),

    /// Staking management: stake, unstake, info.
    #[command(subcommand)]
    Stake(StakeCmd),

    /// Display node connection status and version info.
    Status,

    /// Display the Reef Metagraph (network state).
    Metagraph,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Init => commands::init::run().await?,
        Commands::Wallet(cmd) => commands::wallet::run(cmd).await?,
        Commands::Polyp(cmd) => commands::polyp::run(cmd).await?,
        Commands::Query(cmd) => commands::query::run(cmd).await?,
        Commands::Stake(cmd) => commands::stake::run(cmd).await?,
        Commands::Status => commands::status::run().await?,
        Commands::Metagraph => commands::metagraph::run().await?,
    }

    Ok(())
}
