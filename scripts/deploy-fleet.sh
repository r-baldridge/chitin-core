#!/usr/bin/env bash
# deploy-fleet.sh — Deploy chitin binaries + configs to a node fleet.
#
# Reads configs/fleet.conf for device info, generates per-node configs,
# then distributes binaries and configs to each Linux node via SSH.
# Prints manual instructions for macOS and Docker nodes.
#
# Usage:
#   ./scripts/deploy-fleet.sh [--config-only] [--skip-generate]
#
# Prerequisites:
#   - configs/fleet.conf filled in with real node IPs (see fleet.conf.example)
#   - SSH key access to all SSH-target nodes
#   - Pre-built binaries for each architecture in your fleet:
#       - x86_64-linux:   target/release/chitin-daemon, target/release/chitin
#       - aarch64-linux:  target/aarch64-linux/chitin-daemon (or set AARCH64_BIN_DIR)
#       - aarch64-darwin: build locally on macOS host
#       - x86_64-docker:  docker compose build

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_DIR="$(dirname "$SCRIPT_DIR")"
FLEET_ENV="$REPO_DIR/configs/fleet.conf"
TRIAL_DIR="$REPO_DIR/configs/trial"
SYSTEMD_SERVICE="$SCRIPT_DIR/chitin-daemon.service"

CONFIG_ONLY=false
SKIP_GENERATE=false

for arg in "$@"; do
    case "$arg" in
        --config-only) CONFIG_ONLY=true ;;
        --skip-generate) SKIP_GENERATE=true ;;
        *) echo "Unknown option: $arg"; exit 1 ;;
    esac
done

# --- Validate fleet.conf ---
if [ ! -f "$FLEET_ENV" ]; then
    echo "ERROR: $FLEET_ENV not found."
    echo "Copy configs/fleet.conf.example to configs/fleet.conf and fill in your node IPs."
    exit 1
fi

# --- Parse fleet.conf ---
declare -a LABELS=()
declare -a IPS=()
declare -a SSH_TARGETS=()
declare -a ARCHS=()

while IFS= read -r line; do
    # Skip comments and empty lines
    [[ "$line" =~ ^[[:space:]]*# ]] && continue
    [[ -z "${line// /}" ]] && continue

    read -r label ip ssh_target arch <<< "$line"
    LABELS+=("$label")
    IPS+=("$ip")
    SSH_TARGETS+=("$ssh_target")
    ARCHS+=("$arch")
done < "$FLEET_ENV"

NODE_COUNT=${#LABELS[@]}
echo "=== Chitin Fleet Deployment ==="
echo "Nodes: $NODE_COUNT"
echo ""

# --- Step 1: Generate per-node configs ---
if [ "$SKIP_GENERATE" = false ]; then
    echo "[1/3] Generating per-node configs..."
    GEN_ARGS=()
    for i in "${!LABELS[@]}"; do
        label=$(echo "${LABELS[$i]}" | tr '[:upper:]' '[:lower:]')
        GEN_ARGS+=("${label}=${IPS[$i]}")
    done
    "$SCRIPT_DIR/generate-configs.sh" "${GEN_ARGS[@]}"
    echo ""
else
    echo "[1/3] Skipping config generation (--skip-generate)"
    echo ""
fi

# --- Step 2: Deploy to SSH-accessible nodes ---
echo "[2/3] Deploying to nodes..."
echo ""

MANUAL_NODES=()

for i in "${!LABELS[@]}"; do
    label="${LABELS[$i]}"
    ip="${IPS[$i]}"
    ssh_target="${SSH_TARGETS[$i]}"
    arch="${ARCHS[$i]}"
    label_lower=$(echo "$label" | tr '[:upper:]' '[:lower:]')
    config_file="$TRIAL_DIR/node-${label_lower}.toml"

    if [ ! -f "$config_file" ]; then
        echo "  WARNING: Config not found: $config_file — skipping $label"
        continue
    fi

    # --- Docker nodes: manual instructions ---
    if [ "$ssh_target" = "docker" ]; then
        MANUAL_NODES+=("DOCKER:$label:$ip:$arch")
        continue
    fi

    # --- Local/macOS nodes: manual instructions ---
    if [ "$ssh_target" = "local" ]; then
        MANUAL_NODES+=("LOCAL:$label:$ip:$arch")
        continue
    fi

    # --- SSH-accessible Linux nodes ---
    echo "  Deploying to $label ($ssh_target, $arch)..."

    # Determine binary paths based on architecture
    case "$arch" in
        x86_64-linux)
            DAEMON_BIN="$REPO_DIR/target/release/chitin-daemon"
            CLI_BIN="$REPO_DIR/target/release/chitin"
            ;;
        aarch64-linux)
            # ARM binaries — expected to be in a known location
            # Built on Jetson#1 and copied here, or set AARCH64_BIN_DIR
            AARCH64_BIN_DIR="${AARCH64_BIN_DIR:-$REPO_DIR/target/aarch64-linux}"
            DAEMON_BIN="$AARCH64_BIN_DIR/chitin-daemon"
            CLI_BIN="$AARCH64_BIN_DIR/chitin"
            ;;
        *)
            echo "    WARNING: Unknown arch '$arch' for $label — skipping"
            continue
            ;;
    esac

    if [ "$CONFIG_ONLY" = false ]; then
        if [ ! -f "$DAEMON_BIN" ] || [ ! -f "$CLI_BIN" ]; then
            echo "    WARNING: Binaries not found for $arch:"
            echo "      Expected: $DAEMON_BIN"
            echo "      Expected: $CLI_BIN"
            echo "    Deploying config only."
        else
            echo "    Copying binaries..."
            scp -q "$DAEMON_BIN" "$CLI_BIN" "${ssh_target}:/tmp/"
            ssh -q "$ssh_target" "sudo mv /tmp/chitin-daemon /tmp/chitin /usr/local/bin/ && sudo chmod +x /usr/local/bin/chitin-daemon /usr/local/bin/chitin"
        fi
    fi

    echo "    Copying config..."
    ssh -q "$ssh_target" "mkdir -p ~/.chitin"
    scp -q "$config_file" "${ssh_target}:~/.chitin/config.toml"

    # Copy systemd service if available
    if [ -f "$SYSTEMD_SERVICE" ]; then
        echo "    Installing systemd service..."
        scp -q "$SYSTEMD_SERVICE" "${ssh_target}:/tmp/chitin-daemon.service"
        ssh -q "$ssh_target" "sudo mv /tmp/chitin-daemon.service /etc/systemd/system/ && sudo systemctl daemon-reload && sudo systemctl enable chitin-daemon"
    fi

    # Start/restart the daemon
    if [ "$CONFIG_ONLY" = false ]; then
        echo "    Starting chitin-daemon..."
        ssh -q "$ssh_target" "sudo systemctl restart chitin-daemon" 2>/dev/null || \
            ssh -q "$ssh_target" "nohup chitin-daemon --config ~/.chitin/config.toml > ~/.chitin/daemon.log 2>&1 &"
    else
        echo "    Restarting chitin-daemon..."
        ssh -q "$ssh_target" "sudo systemctl restart chitin-daemon" 2>/dev/null || true
    fi

    echo "    Done: $label"
    echo ""
done

# --- Step 3: Print manual instructions ---
if [ ${#MANUAL_NODES[@]} -gt 0 ]; then
    echo "[3/3] Manual setup required for the following nodes:"
    echo ""

    for entry in "${MANUAL_NODES[@]}"; do
        IFS=':' read -r type label ip arch <<< "$entry"
        label_lower=$(echo "$label" | tr '[:upper:]' '[:lower:]')

        if [ "$type" = "LOCAL" ]; then
            echo "  === $label (macOS / local build) ==="
            echo "  1. Build locally:"
            echo "     cd $REPO_DIR && cargo build --release --bin chitin-daemon --bin chitin"
            echo "  2. Copy config:"
            echo "     cp configs/trial/node-${label_lower}.toml ~/.chitin/config.toml"
            echo "  3. Start daemon:"
            echo "     ./target/release/chitin-daemon --config ~/.chitin/config.toml"
            echo ""
        elif [ "$type" = "DOCKER" ]; then
            echo "  === $label (Docker) ==="
            echo "  1. Copy config to the Docker directory:"
            echo "     cp configs/trial/node-${label_lower}.toml docker/node-config.toml"
            echo "  2. Build and start:"
            echo "     cd docker && docker compose -f docker-compose.trial.yml up -d --build"
            echo ""
        fi
    done
fi

echo "=== Fleet deployment complete ==="
echo ""
echo "Verify all nodes:"
echo "  ./scripts/verify-trial.sh \\"
for i in "${!IPS[@]}"; do
    if [ "$i" -lt $((NODE_COUNT - 1)) ]; then
        echo "    http://${IPS[$i]}:50051 \\"
    else
        echo "    http://${IPS[$i]}:50051"
    fi
done
