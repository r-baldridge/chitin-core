# chitin-core

MVP implementation of the [Chitin Protocol](https://github.com/r-baldridge/reefipedia) (Reefipedia).

## Structure

Rust workspace with 12 crates:

| Crate | Status | Description |
|-------|--------|-------------|
| `chitin-core` | Implemented | Core types, traits, crypto, error handling |
| `chitin-store` | Implemented | RocksDB storage, in-memory vector index, bloom filters |
| `chitin-verify` | Implemented | Placeholder proof generation/verification (SP1 in Phase 3) |
| `chitin-economics` | Implemented | $CTN token, emission, staking, rewards, slashing, treasury |
| `chitin-rpc` | Implemented | JSON-RPC over tonic gRPC server with 33 endpoints |
| `chitin-daemon` | Implemented | Node binary (Coral/Tide/Hybrid modes) |
| `chitin-cli` | Implemented | CLI tool for node management and interaction |
| `chitin-p2p` | Stub | Peer-to-peer networking (Phase 2) |
| `chitin-consensus` | Stub | Yuma-Semantic Consensus (Phase 2) |
| `chitin-reputation` | Stub | Trust matrix and OpenRank integration (Phase 2) |
| `chitin-drift` | Stub | Semantic drift detection and molting (Phase 2) |
| `chitin-sync` | Stub | Set reconciliation and bloom filter sync (Phase 2) |

Supporting files:
- `protos/` - Protocol Buffer definitions (6 .proto files)
- `configs/` - YAML configurations (models, consensus params, economics)
- `docker/` - Dockerfile, Dockerfile.dev, docker-compose.yml (Qdrant + IPFS + daemon)
- `zk-circuits/` - SP1 guest entrypoint scaffold
- `sdk/python/` - Python SDK with LangChain adapter

## Build

```bash
cargo check --workspace   # verify compilation
cargo test --workspace    # run tests (81 passing)
```

## Run

```bash
# Start the daemon
cargo run -p chitin-daemon -- --node-type coral

# Use the CLI
cargo run -p chitin-cli -- status
cargo run -p chitin-cli -- init
cargo run -p chitin-cli -- wallet create
```

## Spec

The full protocol specification lives in the public repo: [reefipedia/ARCHITECTURE.md](https://github.com/r-baldridge/reefipedia/blob/main/ARCHITECTURE.md)
