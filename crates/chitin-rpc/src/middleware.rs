// crates/chitin-rpc/src/middleware.rs
//
// Middleware for the RPC server: logging interceptor and rate limiter.
//
// Phase 1: Basic logging. Phase 2+ will add authentication, rate limiting,
// and request validation.

use tonic::{Request, Status};

/// Logging interceptor for tonic gRPC requests.
///
/// Logs the URI and metadata of each incoming request using the `tracing` crate.
/// In Phase 2+ this will also extract and validate auth tokens.
pub fn logging_interceptor(req: Request<()>) -> Result<Request<()>, Status> {
    tracing::info!(
        "Incoming RPC request: {:?}",
        req.metadata()
    );
    Ok(req)
}

/// Rate limiter stub for the RPC server.
///
/// Phase 1: No actual rate limiting is enforced. This struct exists as a
/// placeholder for the Phase 2 implementation which will use token bucket
/// or sliding window algorithms.
#[derive(Debug, Clone)]
pub struct RateLimiter {
    /// Maximum requests per second per client.
    pub max_rps: u32,
    /// Burst size (max requests allowed in a burst).
    pub burst_size: u32,
}

impl RateLimiter {
    /// Create a new rate limiter with the given parameters.
    pub fn new(max_rps: u32, burst_size: u32) -> Self {
        Self {
            max_rps,
            burst_size,
        }
    }

    /// Check whether a request from the given client should be allowed.
    ///
    /// Phase 1 stub: Always returns true (no rate limiting).
    pub fn check_rate_limit(&self, _client_id: &str) -> bool {
        // Phase 2: Implement token bucket or sliding window rate limiting
        true
    }
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self::new(100, 200)
    }
}
