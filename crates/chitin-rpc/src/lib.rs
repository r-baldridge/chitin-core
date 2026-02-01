// crates/chitin-rpc/src/lib.rs
//
// chitin-rpc: gRPC/JSON-RPC server and handlers for the Chitin Protocol.
//
// Provides a tonic-based RPC server with handlers for all API endpoints
// defined in ARCHITECTURE.md Section 10. Phase 1 uses JSON-based RPC
// over tonic rather than full protobuf codegen.

pub mod handlers;
pub mod middleware;
pub mod server;

// Re-export the main server type for ergonomic access.
pub use server::ChitinRpcServer;
pub use server::RpcConfig;
