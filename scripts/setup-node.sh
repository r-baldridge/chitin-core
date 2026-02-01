#!/usr/bin/env bash
# setup-node.sh — Provision a VM for the Chitin distributed trial.
#
# Usage: curl -sSL <raw-url>/setup-node.sh | bash
# Or:    bash setup-node.sh
#
# Tested on: Ubuntu 22.04 LTS

set -euo pipefail

echo "=== Chitin Node Setup ==="

# --- System dependencies ---
echo "[1/5] Installing system dependencies..."
sudo apt-get update -qq
sudo apt-get install -y -qq \
    build-essential \
    pkg-config \
    libssl-dev \
    libclang-dev \
    clang \
    git \
    curl

# --- Rust toolchain ---
if ! command -v rustc &>/dev/null; then
    echo "[2/5] Installing Rust toolchain..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
else
    echo "[2/5] Rust already installed: $(rustc --version)"
fi

# --- Clone / update repo ---
REPO_DIR="$HOME/chitin-core"
if [ -d "$REPO_DIR" ]; then
    echo "[3/5] Updating existing repo..."
    cd "$REPO_DIR"
    git pull --ff-only || echo "WARNING: git pull failed, using existing checkout"
else
    echo "[3/5] Cloning chitin-core..."
    # Replace with your actual repo URL
    git clone https://github.com/YOUR_ORG/chitin-core.git "$REPO_DIR"
    cd "$REPO_DIR"
fi

# --- Build ---
echo "[4/5] Building release binaries..."
cargo build --release --bin chitin-daemon --bin chitin

# --- Install ---
echo "[5/5] Installing binaries..."
sudo cp target/release/chitin-daemon /usr/local/bin/
sudo cp target/release/chitin /usr/local/bin/

# --- Initialize config ---
chitin init

echo ""
echo "=== Setup Complete ==="
echo ""
echo "Next steps:"
echo "  1. Edit ~/.chitin/config.toml:"
echo "     - Set rpc_host = \"0.0.0.0\""
echo "     - Set self_url = \"http://YOUR_PUBLIC_IP:50051\""
echo "     - Set peers = [\"http://PEER1:50051\", ...]"
echo "  2. Open firewall: sudo ufw allow 50051/tcp"
echo "  3. Start daemon: chitin-daemon --config ~/.chitin/config.toml"
echo "     Or install systemd service: sudo cp scripts/chitin-daemon.service /etc/systemd/system/"
echo "  4. Verify: chitin status --rpc http://localhost:50051"
