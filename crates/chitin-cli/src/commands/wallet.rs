// crates/chitin-cli/src/commands/wallet.rs
//
// `chitin wallet {create, import, export}` — key management commands.

use chitin_core::crypto::Keypair;
use clap::Subcommand;
use std::fs;
use std::path::PathBuf;

/// Wallet management subcommands.
#[derive(Debug, Subcommand)]
pub enum WalletCmd {
    /// Generate a new ed25519 keypair.
    Create,
    /// Import a keypair from a file.
    Import {
        /// Path to the secret key file (hex-encoded).
        #[arg(long)]
        path: String,
    },
    /// Export the current public key.
    Export,
}

/// Run the wallet subcommand.
pub async fn run(cmd: &WalletCmd) -> Result<(), Box<dyn std::error::Error>> {
    match cmd {
        WalletCmd::Create => create_wallet().await,
        WalletCmd::Import { path } => import_wallet(path).await,
        WalletCmd::Export => export_wallet().await,
    }
}

async fn create_wallet() -> Result<(), Box<dyn std::error::Error>> {
    let keypair = Keypair::generate();
    let pubkey = keypair.public_key_bytes();
    let pubkey_hex = hex_encode(&pubkey);

    let keys_dir = get_keys_dir()?;
    fs::create_dir_all(&keys_dir)?;

    let signing_key_bytes = keypair.signing_key.to_bytes();
    let secret_path = keys_dir.join("coldkey.secret");
    let pub_path = keys_dir.join("coldkey.pub");

    fs::write(&secret_path, hex_encode(&signing_key_bytes))?;
    fs::write(&pub_path, &pubkey_hex)?;

    println!("Wallet created successfully.");
    println!("  Public key (coldkey): {}", pubkey_hex);
    println!("  Saved to: {}", pub_path.display());
    println!();
    println!("IMPORTANT: Back up your secret key file securely.");
    println!("  Secret key: {}", secret_path.display());

    Ok(())
}

async fn import_wallet(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let contents = fs::read_to_string(path)?;
    let trimmed = contents.trim();

    // Validate that it's valid hex and 32 bytes.
    if trimmed.len() != 64 {
        return Err("Expected 64-character hex-encoded secret key".into());
    }

    let keys_dir = get_keys_dir()?;
    fs::create_dir_all(&keys_dir)?;

    let dest = keys_dir.join("coldkey.secret");
    fs::write(&dest, trimmed)?;

    println!("Imported secret key from: {}", path);
    println!("Saved to: {}", dest.display());

    Ok(())
}

async fn export_wallet() -> Result<(), Box<dyn std::error::Error>> {
    let keys_dir = get_keys_dir()?;
    let pub_path = keys_dir.join("coldkey.pub");

    if pub_path.exists() {
        let pubkey = fs::read_to_string(&pub_path)?;
        println!("Coldkey public key: {}", pubkey.trim());
    } else {
        println!("No wallet found. Run `chitin wallet create` first.");
    }

    let hotkey_path = keys_dir.join("hotkey.pub");
    if hotkey_path.exists() {
        let hotkey = fs::read_to_string(&hotkey_path)?;
        println!("Hotkey public key:  {}", hotkey.trim());
    }

    Ok(())
}

fn get_keys_dir() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let home = dirs::home_dir().ok_or("Could not determine home directory")?;
    Ok(home.join(".chitin").join("keys"))
}

fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}
