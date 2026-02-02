# chitin-core

Rust implementation of the [Chitin Protocol](https://github.com/r-baldridge/reefipedia) (Reefipedia) — a decentralized semantic knowledge network where knowledge is produced, validated, and hardened through stake-weighted consensus.

## Current Status: Phase 4

- **Phase 1** — Local MVP: core types, storage, RPC server, daemon, CLI
- **Phase 2** — Algorithm crates: identity, consensus, reputation, scoring, drift detection, sync
- **Phase 3** — Stub resolution: all `todo!()` stubs replaced with working implementations
- **Phase 4** (current) — Wired algorithms into daemon: epoch-driven validation, consensus runner, hardening pipeline, 11 of 21 RPC handlers connected to live state

## Architecture

Knowledge flows through the **Polyp lifecycle**:

```
Draft → Soft → UnderReview → Approved → Hardened
                    ↓
                 Rejected
```

**Node types:**
- **Coral** — Knowledge producers (embed text, generate ZK proofs, submit Polyps)
- **Tide** — Validators (score Polyps across 5 dimensions, participate in consensus)
- **Hybrid** — Both producer and validator

**Consensus:** Yuma-Semantic Consensus adapts Bittensor's Yuma Consensus for semantic knowledge validation — stake-weighted median scoring, bond EMA updates, incentive/dividend computation.

## Crates

| Crate | Description |
|-------|-------------|
| `chitin-core` | Core types (Polyp, Identity, Metagraph), traits, Ed25519 crypto, error handling |
| `chitin-store` | RocksDB persistent storage, IPFS client, hardened store, HNSW vector index, Bloom filters |
| `chitin-verify` | ZK proof generation and verification (SP1 scaffold, placeholder verifier) |
| `chitin-economics` | $CTN token (21M max, 9 decimals), emission with halving, staking, rewards, slashing, treasury |
| `chitin-consensus` | Yuma-Semantic Consensus, multi-dimensional scoring, weight/bond matrices, epoch management, hardening |
| `chitin-reputation` | Trust matrix with EigenTrust-style global trust, per-domain scoring, decay |
| `chitin-drift` | Semantic drift detection, embedding alignment, model molting |
| `chitin-sync` | Vector Bloom Filter sync, set reconciliation, domain classification |
| `chitin-p2p` | P2P networking stubs (libp2p transport, discovery, gossip) |
| `chitin-rpc` | JSON-RPC over tonic gRPC — 33 endpoints across 11 handler modules |
| `chitin-daemon` | Node binary with epoch scheduler, TideNode scoring pipeline, consensus runner, hardening pipeline |
| `chitin-cli` | CLI: `init`, `wallet`, `polyp`, `query`, `stake`, `status`, `metagraph` |

### Supporting Files

- `protos/` — Protocol Buffer definitions (6 `.proto` files)
- `configs/` — YAML/TOML configs (model registry, consensus params, economics, trial node)
- `docker/` — Dockerfile, docker-compose (Qdrant + IPFS + daemon)
- `sdk/python/` — Python SDK with gRPC client, types, and LangChain adapter
- `zk-circuits/` — SP1 guest entrypoint scaffold
- `scripts/` — Node setup, fleet deployment, config generation

## Build & Test

```bash
cargo check --workspace          # verify compilation
cargo test --workspace           # run all tests (184 passing)
cargo build --release            # build release binaries
```

## Run

```bash
# Start the daemon
cargo run -p chitin-daemon -- --node-type coral
cargo run -p chitin-daemon -- --node-type tide
cargo run -p chitin-daemon -- --node-type hybrid

# CLI
cargo run -p chitin-cli -- init
cargo run -p chitin-cli -- wallet create
cargo run -p chitin-cli -- polyp create --text "Knowledge content"
cargo run -p chitin-cli -- query "search terms"
cargo run -p chitin-cli -- status
cargo run -p chitin-cli -- metagraph
```

## Docker

```bash
docker build -t chitin-daemon:latest -f docker/Dockerfile .
docker-compose -f docker/docker-compose.yml up -d    # daemon + Qdrant + IPFS
```

## Python SDK

```bash
cd sdk/python && pip install -e .
```

```python
from chitin import ChitinClient

with ChitinClient(host="localhost", port=50051) as client:
    polyp_id = client.submit_polyp("Knowledge text")
    results = client.search("query")
```

## Phase 5 (Next)

Remaining work: wallet key management, $CTN staking/unstaking, admin config hot-reload, and wiring the P2P networking layer for multi-node operation.

## Spec

Full protocol specification: [reefipedia/ARCHITECTURE.md](https://github.com/r-baldridge/reefipedia/blob/main/ARCHITECTURE.md)

## License

Apache-2.0 OR MIT
