// crates/chitin-cli/src/commands/stake.rs
//
// `chitin stake {stake, unstake, info}` — staking management commands.
//
// Phase 1: Print placeholder messages. Real staking in Phase 3.

use clap::Subcommand;

/// Staking subcommands.
#[derive(Debug, Subcommand)]
pub enum StakeCmd {
    /// Stake $CTN tokens to a node.
    Stake {
        /// Amount of $CTN to stake.
        #[arg(long)]
        amount: u64,
        /// Target node hotkey (hex).
        #[arg(long)]
        target: Option<String>,
    },
    /// Begin unstaking $CTN tokens (starts cooldown period).
    Unstake {
        /// Amount of $CTN to unstake.
        #[arg(long)]
        amount: u64,
    },
    /// Show staking information for the current node.
    Info,
}

/// Run the stake subcommand.
pub async fn run(cmd: &StakeCmd) -> Result<(), Box<dyn std::error::Error>> {
    match cmd {
        StakeCmd::Stake { amount, target } => {
            println!("Staking {} CTN", amount);
            if let Some(t) = target {
                println!("  Target node: {}", t);
            } else {
                println!("  Target: self (own node)");
            }
            println!();
            println!("Staking not yet implemented (Phase 3).");
        }
        StakeCmd::Unstake { amount } => {
            println!("Unstaking {} CTN", amount);
            println!();
            println!("Unstaking not yet implemented (Phase 3).");
            println!("Cooldown period: ~24-72 hours depending on node type.");
        }
        StakeCmd::Info => {
            println!("Staking Information");
            println!("-------------------");
            println!("  Staked:       0 CTN (placeholder)");
            println!("  Delegated:    0 CTN (placeholder)");
            println!("  Cooldown:     None");
            println!();
            println!("Note: Phase 1 placeholder. Real staking info in Phase 3.");
        }
    }

    Ok(())
}
