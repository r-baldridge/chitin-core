// crates/chitin-rpc/src/server.rs
//
// RPC server setup: ChitinRpcServer and RpcConfig.
//
// Phase 1: Uses a JSON-RPC-over-gRPC approach. A single tonic unary service
// accepts JSON-encoded requests with a method field, dispatches to the
// appropriate handler, and returns JSON-encoded responses.
//
// This avoids the need for proto codegen while still using tonic's server
// infrastructure for transport, streaming, and middleware.

use std::sync::Arc;
use std::time::Instant;

use http_body::Body as HttpBody;
use http_body_util::BodyExt;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tonic::transport::Server;
use tonic::Status;

use chitin_consensus::bonds::BondMatrix;
use chitin_consensus::epoch::EpochManager;
use chitin_consensus::metagraph::MetagraphManager;
use chitin_consensus::weights::WeightMatrix;
use chitin_consensus::yuma::ConsensusResult;
use chitin_core::identity::NodeIdentity;
use chitin_store::{HardenedStore, InMemoryVectorIndex, RocksStore};

use crate::handlers;
use crate::middleware;

/// Callback type for broadcasting a polyp to peers after creation.
/// The daemon provides this closure to wire gossip into the RPC layer
/// without the RPC crate depending on the daemon's PeerRegistry.
pub type GossipCallback =
    Arc<dyn Fn(chitin_core::polyp::Polyp) + Send + Sync>;

// ---------------------------------------------------------------------------
// RpcConfig
// ---------------------------------------------------------------------------

/// Configuration for the RPC server.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcConfig {
    /// Host to bind to (e.g., "127.0.0.1" or "0.0.0.0").
    pub host: String,
    /// Port to listen on.
    pub port: u16,
}

impl Default for RpcConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 50051,
        }
    }
}

// ---------------------------------------------------------------------------
// JSON-RPC Envelope
// ---------------------------------------------------------------------------

/// A JSON-RPC-style request envelope.
/// The client sends a method name and a JSON params payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    /// The RPC method to invoke (e.g., "polyp/submit", "query/search").
    pub method: String,
    /// JSON-encoded parameters for the method.
    pub params: serde_json::Value,
}

/// A JSON-RPC-style response envelope.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    /// Whether the request succeeded.
    pub success: bool,
    /// The result data (if success).
    pub result: Option<serde_json::Value>,
    /// Error message (if not success).
    pub error: Option<String>,
}

// ---------------------------------------------------------------------------
// ChitinRpcServer
// ---------------------------------------------------------------------------

/// The main RPC server for the Chitin Protocol.
///
/// Holds Arc references to shared state (store, vector index, etc.)
/// and exposes a tonic-based gRPC server with JSON-RPC dispatching.
#[derive(Clone)]
pub struct ChitinRpcServer {
    /// Server configuration.
    config: RpcConfig,
    /// RocksDB-backed Polyp store.
    store: Arc<RocksStore>,
    /// In-memory vector index for ANN search.
    index: Arc<InMemoryVectorIndex>,
    /// Rate limiter (Phase 1: stub).
    #[allow(dead_code)]
    rate_limiter: middleware::RateLimiter,
    /// Optional callback to broadcast a newly created polyp to peers.
    gossip_callback: Option<GossipCallback>,
    /// Number of configured peers.
    peer_count: usize,
    /// Configured peer URLs.
    peer_urls: Vec<String>,
    /// Node identity for provenance and announce responses (Phase 2).
    node_identity: Option<NodeIdentity>,
    /// Signing key for polyp signing (Phase 2).
    signing_key: Option<[u8; 32]>,
    /// This node's publicly reachable URL.
    self_url: Option<String>,
    // Phase 4: Shared consensus/epoch state
    /// Epoch manager for epoch status queries.
    epoch_manager: Option<Arc<RwLock<EpochManager>>>,
    /// Last consensus result for result queries.
    last_consensus_result: Option<Arc<RwLock<Option<ConsensusResult>>>>,
    /// Weight matrix for weight queries and score submission.
    weight_matrix: Option<Arc<RwLock<WeightMatrix>>>,
    /// Bond matrix for bond queries.
    bond_matrix: Option<Arc<RwLock<BondMatrix>>>,
    /// Metagraph manager for metagraph queries.
    metagraph_manager: Option<Arc<RwLock<MetagraphManager>>>,
    /// Hardened store for CID-based retrieval.
    hardened_store: Option<Arc<HardenedStore>>,
    /// Daemon start time for uptime calculation.
    start_time: Option<Instant>,
}

impl std::fmt::Debug for ChitinRpcServer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ChitinRpcServer")
            .field("config", &self.config)
            .field("gossip_enabled", &self.gossip_callback.is_some())
            .finish()
    }
}

impl ChitinRpcServer {
    /// Create a new ChitinRpcServer.
    ///
    /// # Arguments
    /// * `config` - Server configuration (host, port).
    /// * `store` - Shared RocksDB store for Polyp persistence.
    /// * `index` - Shared in-memory vector index for ANN search.
    pub fn new(
        config: RpcConfig,
        store: Arc<RocksStore>,
        index: Arc<InMemoryVectorIndex>,
    ) -> Self {
        Self {
            config,
            store,
            index,
            rate_limiter: middleware::RateLimiter::default(),
            gossip_callback: None,
            peer_count: 0,
            peer_urls: Vec::new(),
            node_identity: None,
            signing_key: None,
            self_url: None,
            epoch_manager: None,
            last_consensus_result: None,
            weight_matrix: None,
            bond_matrix: None,
            metagraph_manager: None,
            hardened_store: None,
            start_time: None,
        }
    }

    /// Set the gossip callback for broadcasting polyps to peers.
    pub fn with_gossip_callback(mut self, callback: GossipCallback) -> Self {
        self.gossip_callback = Some(callback);
        self
    }

    /// Set peer information for health/peers endpoints.
    pub fn with_peer_info(mut self, peer_urls: Vec<String>) -> Self {
        self.peer_count = peer_urls.len();
        self.peer_urls = peer_urls;
        self
    }

    /// Set the node identity and optional signing key for provenance and polyp signing.
    pub fn with_identity(mut self, identity: NodeIdentity, signing_key: Option<[u8; 32]>) -> Self {
        self.node_identity = Some(identity);
        self.signing_key = signing_key;
        self
    }

    /// Set this node's publicly reachable URL for announce responses.
    pub fn with_self_url(mut self, self_url: Option<String>) -> Self {
        self.self_url = self_url;
        self
    }

    /// Set the shared epoch manager for epoch status queries.
    pub fn with_epoch_manager(mut self, em: Arc<RwLock<EpochManager>>) -> Self {
        self.epoch_manager = Some(em);
        self
    }

    /// Set the shared consensus result for result queries.
    pub fn with_consensus_result(mut self, cr: Arc<RwLock<Option<ConsensusResult>>>) -> Self {
        self.last_consensus_result = Some(cr);
        self
    }

    /// Set the shared weight matrix for weight queries and score submission.
    pub fn with_weight_matrix(mut self, wm: Arc<RwLock<WeightMatrix>>) -> Self {
        self.weight_matrix = Some(wm);
        self
    }

    /// Set the shared bond matrix for bond queries.
    pub fn with_bond_matrix(mut self, bm: Arc<RwLock<BondMatrix>>) -> Self {
        self.bond_matrix = Some(bm);
        self
    }

    /// Set the shared metagraph manager for metagraph queries.
    pub fn with_metagraph_manager(mut self, mm: Arc<RwLock<MetagraphManager>>) -> Self {
        self.metagraph_manager = Some(mm);
        self
    }

    /// Set the hardened store for CID-based retrieval.
    pub fn with_hardened_store(mut self, hs: Option<Arc<HardenedStore>>) -> Self {
        self.hardened_store = hs;
        self
    }

    /// Set the daemon start time for uptime calculation.
    pub fn with_start_time(mut self, st: Instant) -> Self {
        self.start_time = Some(st);
        self
    }

    /// Start the RPC server and listen for requests.
    ///
    /// This binds to the configured address and serves requests until
    /// the process is terminated.
    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        let addr = format!("{}:{}", self.config.host, self.config.port).parse()?;

        tracing::info!("Chitin RPC server starting on {}", addr);

        let service = ChitinServiceImpl {
            store: self.store.clone(),
            index: self.index.clone(),
            gossip_callback: self.gossip_callback.clone(),
            peer_count: self.peer_count,
            peer_urls: self.peer_urls.clone(),
            node_identity: self.node_identity.clone(),
            signing_key: self.signing_key,
            self_url: self.self_url.clone(),
            epoch_manager: self.epoch_manager.clone(),
            last_consensus_result: self.last_consensus_result.clone(),
            weight_matrix: self.weight_matrix.clone(),
            bond_matrix: self.bond_matrix.clone(),
            metagraph_manager: self.metagraph_manager.clone(),
            hardened_store: self.hardened_store.clone(),
            start_time: self.start_time,
        };

        Server::builder()
            .accept_http1(true)
            .add_service(
                tonic::service::interceptor::InterceptedService::new(
                    ChitinJsonRpcServer::new(service),
                    middleware::logging_interceptor,
                ),
            )
            .serve(addr)
            .await?;

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// gRPC Service Definition (manual, no proto codegen)
// ---------------------------------------------------------------------------

/// The internal service implementation that holds shared state
/// and dispatches JSON-RPC calls to the appropriate handler.
#[derive(Clone)]
struct ChitinServiceImpl {
    store: Arc<RocksStore>,
    index: Arc<InMemoryVectorIndex>,
    gossip_callback: Option<GossipCallback>,
    /// Number of configured peers (for health endpoint).
    peer_count: usize,
    /// Configured peer URLs (for peers endpoint).
    peer_urls: Vec<String>,
    /// Node identity for provenance and announce responses (Phase 2).
    node_identity: Option<NodeIdentity>,
    /// Signing key for polyp signing (Phase 2).
    signing_key: Option<[u8; 32]>,
    /// This node's publicly reachable URL.
    self_url: Option<String>,
    // Phase 4: Shared consensus/epoch state
    epoch_manager: Option<Arc<RwLock<EpochManager>>>,
    last_consensus_result: Option<Arc<RwLock<Option<ConsensusResult>>>>,
    weight_matrix: Option<Arc<RwLock<WeightMatrix>>>,
    bond_matrix: Option<Arc<RwLock<BondMatrix>>>,
    metagraph_manager: Option<Arc<RwLock<MetagraphManager>>>,
    hardened_store: Option<Arc<HardenedStore>>,
    start_time: Option<Instant>,
}

impl ChitinServiceImpl {
    /// Dispatch a JSON-RPC request to the appropriate handler based on the method name.
    async fn dispatch(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        let result = match request.method.as_str() {
            // Polyp Management
            "polyp/submit" => {
                let store = self.store.clone();
                let index = self.index.clone();
                let gossip_cb = self.gossip_callback.clone();
                let identity = self.node_identity.clone();
                let sign_key = self.signing_key;
                let req: Result<handlers::polyp::SubmitPolypRequest, _> =
                    serde_json::from_value(request.params);
                match req {
                    Ok(r) => {
                        match handlers::polyp::handle_submit_polyp_with_identity(
                            &store,
                            &index,
                            r,
                            identity.as_ref(),
                            sign_key.as_ref(),
                        ).await {
                            Ok(resp) => {
                                // Trigger gossip broadcast if callback is set.
                                if let Some(cb) = gossip_cb {
                                    if let Ok(Some(polyp)) = chitin_core::traits::PolypStore::get_polyp(
                                        store.as_ref(),
                                        &resp.polyp_id,
                                    )
                                    .await
                                    {
                                        cb(polyp);
                                    }
                                }
                                serde_json::to_value(resp)
                                    .map_err(|e| format!("Failed to serialize response: {}", e))
                            }
                            Err(e) => Err(e),
                        }
                    }
                    Err(e) => Err(format!("Failed to deserialize request: {}", e)),
                }
            }
            "polyp/get" => {
                dispatch_handler(request.params, |r| {
                    let store = self.store.clone();
                    async move { handlers::polyp::handle_get_polyp(&store, r).await }
                })
                .await
            }
            "polyp/list" => {
                dispatch_handler(request.params, |r| {
                    let store = self.store.clone();
                    async move { handlers::polyp::handle_list_polyps(&store, r).await }
                })
                .await
            }
            "polyp/state" => {
                dispatch_handler(request.params, |r| {
                    let store = self.store.clone();
                    async move { handlers::polyp::handle_get_polyp_state(&store, r).await }
                })
                .await
            }
            "polyp/provenance" => {
                dispatch_handler(request.params, |r| {
                    let store = self.store.clone();
                    async move { handlers::polyp::handle_get_polyp_provenance(&store, r).await }
                })
                .await
            }
            "polyp/hardening" => {
                dispatch_handler(request.params, |r| {
                    let store = self.store.clone();
                    async move { handlers::polyp::handle_get_hardening_receipt(&store, r).await }
                })
                .await
            }

            // Query / Retrieval
            "query/search" => {
                dispatch_handler(request.params, |r| {
                    let store = self.store.clone();
                    let index = self.index.clone();
                    async move { handlers::query::handle_semantic_search(&store, &index, r).await }
                })
                .await
            }
            "query/hybrid" => {
                dispatch_handler(request.params, |r| {
                    let store = self.store.clone();
                    let index = self.index.clone();
                    async move { handlers::query::handle_hybrid_search(&store, &index, r).await }
                })
                .await
            }
            "query/cid" => {
                let hardened_store = self.hardened_store.clone();
                dispatch_handler(request.params, |r| {
                    async move { handlers::query::handle_get_by_cid(hardened_store.as_ref(), r).await }
                })
                .await
            }
            "query/explain" => {
                dispatch_handler(request.params, |r| {
                    let store = self.store.clone();
                    async move { handlers::query::handle_explain_result(&store, r).await }
                })
                .await
            }

            // Node
            "node/info" => {
                let identity = self.node_identity.clone();
                let start_time = self.start_time;
                dispatch_handler(request.params, |r| async move {
                    handlers::node::handle_get_node_info(r, identity.as_ref(), start_time).await
                })
                .await
            }
            "node/health" => {
                let peer_count = self.peer_count;
                dispatch_handler(request.params, |r| async move {
                    handlers::node::handle_get_health(r, peer_count).await
                })
                .await
            }
            "node/peers" => {
                let peer_urls = self.peer_urls.clone();
                dispatch_handler(request.params, |r| async move {
                    let peer_data: Vec<handlers::node::PeerInfo> = peer_urls
                        .into_iter()
                        .map(|url| handlers::node::PeerInfo {
                            peer_id: url.clone(),
                            address: url,
                            node_type: None,
                            latency_ms: None,
                        })
                        .collect();
                    handlers::node::handle_get_peers(r, peer_data).await
                })
                .await
            }

            // Wallet
            "wallet/create" => {
                dispatch_handler(request.params, |r| async move {
                    handlers::wallet::handle_create_wallet(r).await
                })
                .await
            }
            "wallet/import" => {
                dispatch_handler(request.params, |r| async move {
                    handlers::wallet::handle_import_wallet(r).await
                })
                .await
            }
            "wallet/balance" => {
                dispatch_handler(request.params, |r| async move {
                    handlers::wallet::handle_get_balance(r).await
                })
                .await
            }
            "wallet/transfer" => {
                dispatch_handler(request.params, |r| async move {
                    handlers::wallet::handle_transfer(r).await
                })
                .await
            }

            // Staking
            "staking/stake" => {
                dispatch_handler(request.params, |r| async move {
                    handlers::staking::handle_stake(r).await
                })
                .await
            }
            "staking/unstake" => {
                dispatch_handler(request.params, |r| async move {
                    handlers::staking::handle_unstake(r).await
                })
                .await
            }
            "staking/info" => {
                dispatch_handler(request.params, |r| async move {
                    handlers::staking::handle_get_stake_info(r).await
                })
                .await
            }

            // Metagraph
            "metagraph/get" => {
                let mm = self.metagraph_manager.clone();
                dispatch_handler(request.params, |r| async move {
                    handlers::metagraph::handle_get_metagraph(r, mm.as_ref()).await
                })
                .await
            }
            "metagraph/node" => {
                let mm = self.metagraph_manager.clone();
                dispatch_handler(request.params, |r| async move {
                    handlers::metagraph::handle_get_node_metrics(r, mm.as_ref()).await
                })
                .await
            }
            "metagraph/weights" => {
                let wm = self.weight_matrix.clone();
                let em = self.epoch_manager.clone();
                dispatch_handler(request.params, |r| async move {
                    handlers::metagraph::handle_get_weights(r, wm.as_ref(), em.as_ref()).await
                })
                .await
            }
            "metagraph/bonds" => {
                let bm = self.bond_matrix.clone();
                let em = self.epoch_manager.clone();
                dispatch_handler(request.params, |r| async move {
                    handlers::metagraph::handle_get_bonds(r, bm.as_ref(), em.as_ref()).await
                })
                .await
            }

            // Validation
            "validation/scores" => {
                let wm = self.weight_matrix.clone();
                let em = self.epoch_manager.clone();
                dispatch_handler(request.params, |r| async move {
                    handlers::validation::handle_submit_scores(r, wm.as_ref(), em.as_ref()).await
                })
                .await
            }
            "validation/epoch" => {
                let em = self.epoch_manager.clone();
                dispatch_handler(request.params, |r| async move {
                    handlers::validation::handle_get_epoch_status(r, em.as_ref()).await
                })
                .await
            }
            "validation/result" => {
                let cr = self.last_consensus_result.clone();
                dispatch_handler(request.params, |r| async move {
                    handlers::validation::handle_get_consensus_result(r, cr.as_ref()).await
                })
                .await
            }

            // Sync
            "sync/status" => {
                let peer_count = self.peer_count;
                dispatch_handler(request.params, |r| async move {
                    handlers::sync::handle_get_sync_status(r, peer_count).await
                })
                .await
            }
            "sync/trigger" => {
                let peer_count = self.peer_count;
                dispatch_handler(request.params, |r| async move {
                    handlers::sync::handle_trigger_sync(r, peer_count).await
                })
                .await
            }

            // Admin
            "admin/config" => {
                dispatch_handler(request.params, |r| async move {
                    handlers::admin::handle_get_config(r).await
                })
                .await
            }
            "admin/config/update" => {
                dispatch_handler(request.params, |r| async move {
                    handlers::admin::handle_update_config(r).await
                })
                .await
            }
            "admin/logs" => {
                dispatch_handler(request.params, |r| async move {
                    handlers::admin::handle_get_logs(r).await
                })
                .await
            }

            // Peer Relay
            "peer/announce" => {
                let self_did = self.node_identity.as_ref().map(|id| id.did.clone());
                let self_url = self.self_url.clone();
                dispatch_handler(request.params, |r| async move {
                    handlers::peer::handle_announce_with_identity(r, self_did, self_url).await
                })
                .await
            }
            "peer/receive_polyp" => {
                dispatch_handler(request.params, |r| {
                    let store = self.store.clone();
                    let index = self.index.clone();
                    async move {
                        handlers::peer::handle_receive_polyp(&store, &index, r).await
                    }
                })
                .await
            }
            "peer/list_polyp_ids" => {
                dispatch_handler(request.params, |r| {
                    let store = self.store.clone();
                    async move {
                        handlers::peer::handle_list_polyp_ids(&store, r).await
                    }
                })
                .await
            }
            "peer/discover" => {
                let peer_urls = self.peer_urls.clone();
                dispatch_handler(request.params, |r| async move {
                    let peer_data: Vec<handlers::peer::DiscoveredPeer> = peer_urls
                        .into_iter()
                        .map(|url| handlers::peer::DiscoveredPeer {
                            url,
                            did: None,
                            alive: false,
                        })
                        .collect();
                    handlers::peer::handle_discover_peers(r, peer_data).await
                })
                .await
            }

            _ => Err(format!("Unknown method: {}", request.method)),
        };

        match result {
            Ok(value) => JsonRpcResponse {
                success: true,
                result: Some(value),
                error: None,
            },
            Err(err) => JsonRpcResponse {
                success: false,
                result: None,
                error: Some(err),
            },
        }
    }
}

/// Generic dispatch helper: deserialize params into a request type,
/// call the handler, and serialize the result to JSON.
async fn dispatch_handler<Req, Resp, F, Fut>(
    params: serde_json::Value,
    handler: F,
) -> Result<serde_json::Value, String>
where
    Req: serde::de::DeserializeOwned,
    Resp: serde::Serialize,
    F: FnOnce(Req) -> Fut,
    Fut: std::future::Future<Output = Result<Resp, String>>,
{
    let request: Req = serde_json::from_value(params)
        .map_err(|e| format!("Failed to deserialize request: {}", e))?;
    let response = handler(request).await?;
    serde_json::to_value(response).map_err(|e| format!("Failed to serialize response: {}", e))
}

// ---------------------------------------------------------------------------
// Tonic Service Wiring
// ---------------------------------------------------------------------------
// We define a single gRPC service with one method: `Call`.
// The request and response are raw bytes (JSON-encoded JsonRpcRequest/Response).
// This avoids proto codegen entirely.

/// The tonic service wrapper. Implements the low-level gRPC service
/// by accepting bytes, deserializing as JSON-RPC, and dispatching.
#[derive(Clone)]
pub struct ChitinJsonRpcServer {
    inner: ChitinServiceImpl,
}

impl std::fmt::Debug for ChitinJsonRpcServer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ChitinJsonRpcServer").finish()
    }
}

impl ChitinJsonRpcServer {
    fn new(inner: ChitinServiceImpl) -> Self {
        Self { inner }
    }
}

// Implement tonic::codegen::Service manually for our JSON-RPC service.
// This is the pattern for defining tonic services without proto codegen.
impl tonic::server::NamedService for ChitinJsonRpcServer {
    const NAME: &'static str = "chitin.rpc.ChitinService";
}

impl<B> tower_service::Service<http::Request<B>> for ChitinJsonRpcServer
where
    B: HttpBody + Send + 'static,
    B::Error: Into<Box<dyn std::error::Error + Send + Sync>> + Send,
    B::Data: Send,
{
    type Response = http::Response<tonic::body::BoxBody>;
    type Error = std::convert::Infallible;
    type Future = std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>> + Send>,
    >;

    fn poll_ready(
        &mut self,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        std::task::Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: http::Request<B>) -> Self::Future {
        let inner = self.inner.clone();

        Box::pin(async move {
            // Read the full request body.
            let body = req.into_body();
            let body_bytes = match collect_body(body).await {
                Ok(b) => b,
                Err(e) => {
                    tracing::error!("Failed to read request body: {}", e);
                    let resp = JsonRpcResponse {
                        success: false,
                        result: None,
                        error: Some(format!("Failed to read request body: {}", e)),
                    };
                    let json = serde_json::to_vec(&resp).unwrap_or_default();
                    return Ok(build_response(json));
                }
            };

            // Deserialize the JSON-RPC request.
            let rpc_request: JsonRpcRequest = match serde_json::from_slice(&body_bytes) {
                Ok(r) => r,
                Err(e) => {
                    let resp = JsonRpcResponse {
                        success: false,
                        result: None,
                        error: Some(format!("Invalid JSON-RPC request: {}", e)),
                    };
                    let json = serde_json::to_vec(&resp).unwrap_or_default();
                    return Ok(build_response(json));
                }
            };

            // Dispatch to the appropriate handler.
            let rpc_response = inner.dispatch(rpc_request).await;
            let json = serde_json::to_vec(&rpc_response).unwrap_or_default();
            Ok(build_response(json))
        })
    }
}

/// Collect the body of an HTTP request into bytes.
async fn collect_body<B>(body: B) -> Result<Vec<u8>, String>
where
    B: HttpBody + Send,
    B::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
    B::Data: Send,
{
    let mut collected = Vec::new();
    let mut body = std::pin::pin!(body);

    loop {
        match std::future::poll_fn(|cx| HttpBody::poll_frame(body.as_mut(), cx)).await {
            Some(Ok(frame)) => {
                if let Ok(data) = frame.into_data() {
                    use bytes::Buf;
                    collected.extend_from_slice(data.chunk());
                }
            }
            Some(Err(e)) => return Err(e.into().to_string()),
            None => break,
        }
    }

    Ok(collected)
}

/// Build an HTTP response with the given JSON body.
fn build_response(json: Vec<u8>) -> http::Response<tonic::body::BoxBody> {
    let body = tonic::body::BoxBody::new(
        http_body_util::Full::new(bytes::Bytes::from(json))
            .map_err(|e| Status::internal(format!("body error: {}", e))),
    );

    http::Response::builder()
        .status(200)
        .header("content-type", "application/json")
        .body(body)
        .unwrap()
}
