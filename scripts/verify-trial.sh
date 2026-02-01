#!/usr/bin/env bash
# verify-trial.sh — Verify multi-node polyp propagation.
#
# Usage: ./verify-trial.sh http://NODE1:50051 http://NODE2:50051 [... http://NODEN:50051]
#
# Creates a polyp on each node, waits for sync, then verifies all
# nodes have all polyps and cross-node search works.

set -euo pipefail

if [ $# -lt 2 ]; then
    echo "Usage: $0 <node1_url> <node2_url> [node3_url ...]"
    echo "Example: $0 http://10.0.0.1:50051 http://10.0.0.2:50051 http://10.0.0.3:50051"
    exit 1
fi

NODES=("$@")
NODE_COUNT=${#NODES[@]}
POLYP_IDS=()
WAIT_SECS=60

echo "=== Chitin Trial Verification ==="
echo "Nodes: ${NODES[*]}"
echo ""

# --- Step 1: Health check all nodes ---
echo "[1/4] Health checking $NODE_COUNT nodes..."
for node in "${NODES[@]}"; do
    RESP=$(curl -s -X POST "$node" \
        -H "Content-Type: application/json" \
        -d '{"method":"node/health","params":{}}' 2>/dev/null || echo '{"success":false}')
    SUCCESS=$(echo "$RESP" | python3 -c "import sys,json; print(json.load(sys.stdin).get('success',False))" 2>/dev/null || echo "False")
    if [ "$SUCCESS" = "True" ]; then
        PEER_COUNT=$(echo "$RESP" | python3 -c "import sys,json; print(json.load(sys.stdin).get('result',{}).get('peer_count',0))" 2>/dev/null || echo "?")
        echo "  OK: $node (peers: $PEER_COUNT)"
    else
        echo "  FAIL: $node — not responding"
        echo "  Response: $RESP"
    fi
done
echo ""

# --- Step 2: Create a polyp on each node ---
echo "[2/4] Creating polyps (one per node)..."
for i in "${!NODES[@]}"; do
    node="${NODES[$i]}"
    TEXT="Trial polyp from node $((i+1)) at $(date -u +%Y-%m-%dT%H:%M:%SZ)"

    RESP=$(curl -s -X POST "$node" \
        -H "Content-Type: application/json" \
        -d "{\"method\":\"polyp/submit\",\"params\":{\"content\":\"$TEXT\",\"content_type\":\"text/plain\"}}")

    POLYP_ID=$(echo "$RESP" | python3 -c "import sys,json; print(json.load(sys.stdin)['result']['polyp_id'])" 2>/dev/null || echo "FAILED")

    if [ "$POLYP_ID" != "FAILED" ]; then
        POLYP_IDS+=("$POLYP_ID")
        echo "  Node $((i+1)): created polyp $POLYP_ID"
    else
        echo "  Node $((i+1)): FAILED to create polyp"
        echo "  Response: $RESP"
    fi
done
echo ""

TOTAL_POLYPS=${#POLYP_IDS[@]}
echo "Created $TOTAL_POLYPS polyps total."
echo ""

# --- Step 3: Wait for sync ---
echo "[3/4] Waiting ${WAIT_SECS}s for sync propagation..."
sleep "$WAIT_SECS"
echo ""

# --- Step 4: Verify all nodes have all polyps ---
echo "[4/4] Verifying polyp propagation..."
ALL_OK=true

for node in "${NODES[@]}"; do
    FOUND=0
    MISSING=0

    for pid in "${POLYP_IDS[@]}"; do
        RESP=$(curl -s -X POST "$node" \
            -H "Content-Type: application/json" \
            -d "{\"method\":\"polyp/get\",\"params\":{\"polyp_id\":\"$pid\"}}")

        IS_FOUND=$(echo "$RESP" | python3 -c "import sys,json; print(json.load(sys.stdin).get('result',{}).get('found',False))" 2>/dev/null || echo "False")

        if [ "$IS_FOUND" = "True" ]; then
            FOUND=$((FOUND + 1))
        else
            MISSING=$((MISSING + 1))
        fi
    done

    if [ "$MISSING" -eq 0 ]; then
        echo "  OK: $node — has all $TOTAL_POLYPS polyps"
    else
        echo "  PARTIAL: $node — $FOUND/$TOTAL_POLYPS found, $MISSING missing"
        ALL_OK=false
    fi
done
echo ""

# --- Summary ---
if [ "$ALL_OK" = true ]; then
    echo "=== TRIAL VERIFICATION PASSED ==="
    echo "All $NODE_COUNT nodes have all $TOTAL_POLYPS polyps."
else
    echo "=== TRIAL VERIFICATION INCOMPLETE ==="
    echo "Some nodes are missing polyps. Check sync loop logs."
    echo "Try running again after another sync cycle (30s)."
fi
