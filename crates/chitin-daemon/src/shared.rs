// crates/chitin-daemon/src/shared.rs
//
// DaemonSharedState: centralized shared mutable state for the Chitin daemon.
//
// Constructed once in main.rs, then injected into daemon tasks (TideNode,
// EpochScheduler, consensus runner) and the RPC server via builder methods.

use std::sync::Arc;
use std::time::Instant;

use tokio::sync::RwLock;

use chitin_consensus::bonds::BondMatrix;
use chitin_consensus::epoch::EpochManager;
use chitin_consensus::metagraph::MetagraphManager;
use chitin_consensus::weights::WeightMatrix;
use chitin_consensus::yuma::ConsensusResult;
use chitin_reputation::trust_matrix::TrustMatrix;
use chitin_store::HardenedStore;

/// Shared mutable state for the daemon, wrapped in Arc<RwLock<>> for
/// safe concurrent access from multiple tokio tasks.
#[derive(Clone)]
pub struct DaemonSharedState {
    /// Epoch lifecycle manager (tracks current epoch + phase).
    pub epoch_manager: Arc<RwLock<EpochManager>>,
    /// Last completed consensus result (None until first epoch completes).
    pub last_consensus_result: Arc<RwLock<Option<ConsensusResult>>>,
    /// Trust matrix: T(from, to) trust values between validators.
    pub trust_matrix: Arc<RwLock<TrustMatrix>>,
    /// Weight matrix: W[validator][coral] scores for the current epoch.
    pub weight_matrix: Arc<RwLock<WeightMatrix>>,
    /// Bond matrix: EMA-smoothed historical weights.
    pub bond_matrix: Arc<RwLock<BondMatrix>>,
    /// Local metagraph snapshot manager.
    pub metagraph_manager: Arc<RwLock<MetagraphManager>>,
    /// Optional hardened store (IPFS-backed immutable storage).
    pub hardened_store: Option<Arc<HardenedStore>>,
    /// Daemon start time for uptime calculation.
    pub start_time: Instant,
}

impl DaemonSharedState {
    /// Create a new DaemonSharedState with the given configuration.
    ///
    /// Initializes all matrices to a default network size of 0 validators
    /// and 0 coral nodes. These will be resized as nodes register.
    pub fn new(blocks_per_epoch: u64, hardened_store: Option<Arc<HardenedStore>>) -> Self {
        Self {
            epoch_manager: Arc::new(RwLock::new(EpochManager::new(blocks_per_epoch))),
            last_consensus_result: Arc::new(RwLock::new(None)),
            trust_matrix: Arc::new(RwLock::new(TrustMatrix::new())),
            weight_matrix: Arc::new(RwLock::new(WeightMatrix::new(0, 0))),
            bond_matrix: Arc::new(RwLock::new(BondMatrix::new(0, 0))),
            metagraph_manager: Arc::new(RwLock::new(MetagraphManager::new())),
            hardened_store,
            start_time: Instant::now(),
        }
    }
}
