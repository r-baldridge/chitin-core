// crates/chitin-cli/src/commands/metagraph.rs
//
// `chitin metagraph` — display the Reef Metagraph (network state).
//
// Phase 1: Print a placeholder table. Real metagraph query in Phase 2+.

use tabled::{Table, Tabled};

/// A row in the metagraph display table.
#[derive(Tabled)]
struct MetagraphRow {
    #[tabled(rename = "UID")]
    uid: u16,
    #[tabled(rename = "Type")]
    node_type: String,
    #[tabled(rename = "Stake")]
    stake: String,
    #[tabled(rename = "Trust")]
    trust: String,
    #[tabled(rename = "Consensus")]
    consensus: String,
    #[tabled(rename = "Incentive")]
    incentive: String,
    #[tabled(rename = "Emission")]
    emission: String,
    #[tabled(rename = "Polyps")]
    polyps: u64,
    #[tabled(rename = "Active")]
    active: String,
}

/// Run the metagraph command.
pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
    println!("Reef Metagraph (Phase 1 placeholder)");
    println!("Epoch: 0  |  Block: 0  |  Total Stake: 0 CTN");
    println!();

    // Phase 1: display a placeholder table with sample data.
    let rows = vec![
        MetagraphRow {
            uid: 0,
            node_type: "Coral".to_string(),
            stake: "0 CTN".to_string(),
            trust: "0.000".to_string(),
            consensus: "0.000".to_string(),
            incentive: "0.000".to_string(),
            emission: "0".to_string(),
            polyps: 0,
            active: "--".to_string(),
        },
        MetagraphRow {
            uid: 1,
            node_type: "Tide".to_string(),
            stake: "0 CTN".to_string(),
            trust: "0.000".to_string(),
            consensus: "0.000".to_string(),
            incentive: "0.000".to_string(),
            emission: "0".to_string(),
            polyps: 0,
            active: "--".to_string(),
        },
    ];

    let table = Table::new(&rows).to_string();
    println!("{}", table);
    println!();
    println!("Note: Phase 1 placeholder data. Real metagraph in Phase 2+.");

    Ok(())
}
