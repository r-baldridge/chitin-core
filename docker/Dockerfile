# Multi-stage build for chitin-daemon + chitin CLI
# Stage 1: Builder
FROM rust:1.83-bookworm AS builder

WORKDIR /build

# Copy workspace manifests first for layer caching
COPY Cargo.toml Cargo.lock ./
COPY crates/ crates/

# Build release binaries (daemon + CLI)
RUN cargo build --release --bin chitin-daemon --bin chitin

# Stage 2: Runtime
FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -u 1000 chitin

# Copy binaries from builder
COPY --from=builder /build/target/release/chitin-daemon /usr/local/bin/chitin-daemon
COPY --from=builder /build/target/release/chitin /usr/local/bin/chitin

# Copy default config files
COPY configs/ /etc/chitin/configs/

# Create data directories
RUN mkdir -p /data/rocksdb /data/ipfs /data/qdrant && \
    chown -R chitin:chitin /data /etc/chitin

USER chitin

# gRPC port
EXPOSE 50051
# IPFS port
EXPOSE 4001

ENV CHITIN_CONFIG_DIR=/etc/chitin/configs
ENV CHITIN_DATA_DIR=/data
ENV RUST_LOG=info

ENTRYPOINT ["chitin-daemon"]
