#!/usr/bin/env bash
# generate-configs.sh — Generate per-node config.toml files for the distributed trial.
#
# Usage:
#   ./scripts/generate-configs.sh label1=ip1 label2=ip2 [label3=ip3 ...]
#
# Example (3-node fleet):
#   ./scripts/generate-configs.sh node1=10.0.0.1 node2=10.0.0.2 node3=10.0.0.3
#
# Output: configs/trial/node-{label}.toml for each node

set -euo pipefail

if [ $# -lt 2 ]; then
    echo "Usage: $0 label1=ip1 label2=ip2 [label3=ip3 ...]"
    echo ""
    echo "Example:"
    echo "  $0 node1=10.0.0.1 node2=10.0.0.2 node3=10.0.0.3"
    exit 1
fi

# Parse label=ip pairs
declare -a LABELS=()
declare -a IPS=()

for arg in "$@"; do
    if [[ "$arg" != *"="* ]]; then
        echo "ERROR: Invalid argument '$arg'. Expected format: label=ip"
        exit 1
    fi
    label="${arg%%=*}"
    ip="${arg#*=}"
    LABELS+=("$label")
    IPS+=("$ip")
done

NODE_COUNT=${#LABELS[@]}
PORT=50051

# Create output directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_DIR="$(dirname "$SCRIPT_DIR")"
OUT_DIR="$REPO_DIR/configs/trial"
mkdir -p "$OUT_DIR"

echo "=== Generating $NODE_COUNT node configs ==="
echo "Output: $OUT_DIR/"
echo ""

for i in "${!LABELS[@]}"; do
    label="${LABELS[$i]}"
    ip="${IPS[$i]}"
    outfile="$OUT_DIR/node-${label}.toml"

    # Build peers list (all nodes except self)
    peers=""
    for j in "${!LABELS[@]}"; do
        if [ "$j" != "$i" ]; then
            if [ -n "$peers" ]; then
                peers="$peers,"$'\n'"    \"http://${IPS[$j]}:${PORT}\""
            else
                peers="    \"http://${IPS[$j]}:${PORT}\""
            fi
        fi
    done

    cat > "$outfile" <<EOF
# Chitin Protocol — Trial Node Configuration
# Node: ${label} (${ip})
# Generated: $(date -u +%Y-%m-%dT%H:%M:%SZ)

node_type = "coral"
data_dir = "~/.chitin/data"
rpc_host = "0.0.0.0"
rpc_port = ${PORT}
p2p_port = 4001
ipfs_api_url = "http://127.0.0.1:5001"
log_level = "info"

self_url = "http://${ip}:${PORT}"

peers = [
${peers}
]
EOF

    echo "  Created: node-${label}.toml (self=${ip}, ${NODE_COUNT}-1 peers)"
done

echo ""
echo "=== Done. ${NODE_COUNT} config files in ${OUT_DIR}/ ==="
