// crates/chitin-rpc/src/handlers/staking.rs
//
// Staking handlers: Stake, Unstake, GetStakeInfo.
// Phase 1: Stub implementations. Phase 3 will implement real staking
// using chitin-economics::StakeManager.

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Stake
// ---------------------------------------------------------------------------

/// Request to stake $CTN to a node.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakeRequest {
    /// Hex-encoded coldkey of the staker.
    pub staker_coldkey: String,
    /// Network UID of the node to stake to.
    pub node_uid: u16,
    /// Amount to stake in rao.
    pub amount_rao: u64,
}

/// Response from a stake operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakeResponse {
    /// Whether the stake was successful.
    pub success: bool,
    /// New total stake for the staker on this node.
    pub new_total_rao: u64,
    /// Human-readable message.
    pub message: String,
}

/// Handle a Stake request.
///
/// Phase 1 stub: Staking is not yet active.
pub async fn handle_stake(_request: StakeRequest) -> Result<StakeResponse, String> {
    // Phase 3: Use chitin_economics::StakeManager to process the stake
    Ok(StakeResponse {
        success: false,
        new_total_rao: 0,
        message: "Phase 1 stub: staking not yet implemented".to_string(),
    })
}

// ---------------------------------------------------------------------------
// Unstake
// ---------------------------------------------------------------------------

/// Request to begin unstaking $CTN from a node.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnstakeRequest {
    /// Hex-encoded coldkey of the staker.
    pub staker_coldkey: String,
    /// Network UID of the node to unstake from.
    pub node_uid: u16,
    /// Amount to unstake in rao. Use 0 for full unstake.
    pub amount_rao: u64,
}

/// Response from an unstake operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnstakeResponse {
    /// Whether the unstake request was accepted.
    pub success: bool,
    /// The block at which the cooldown completes.
    pub cooldown_complete_block: Option<u64>,
    /// Human-readable message.
    pub message: String,
}

/// Handle an Unstake request.
///
/// Phase 1 stub: Unstaking is not yet active.
pub async fn handle_unstake(_request: UnstakeRequest) -> Result<UnstakeResponse, String> {
    // Phase 3: Use chitin_economics::StakeManager to request unstake
    Ok(UnstakeResponse {
        success: false,
        cooldown_complete_block: None,
        message: "Phase 1 stub: unstaking not yet implemented".to_string(),
    })
}

// ---------------------------------------------------------------------------
// GetStakeInfo
// ---------------------------------------------------------------------------

/// Request for staking information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetStakeInfoRequest {
    /// Hex-encoded coldkey to query. If omitted, returns info for all stakers.
    pub coldkey: Option<String>,
    /// Network UID to query. If omitted, returns info for all nodes.
    pub node_uid: Option<u16>,
}

/// Information about a single stake.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakeInfo {
    /// Hex-encoded staker coldkey.
    pub staker_coldkey: String,
    /// Node UID staked to.
    pub node_uid: u16,
    /// Amount staked in rao.
    pub amount_rao: u64,
    /// Amount in CTN (for display).
    pub amount_ctn: f64,
    /// Block at which the stake was created.
    pub staked_at_block: u64,
    /// Whether an unstake is pending.
    pub unstake_pending: bool,
    /// Block at which the cooldown completes (if unstake pending).
    pub cooldown_complete_block: Option<u64>,
}

/// Response containing staking information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetStakeInfoResponse {
    /// List of stake entries matching the query.
    pub stakes: Vec<StakeInfo>,
    /// Total staked amount across all matching entries (in rao).
    pub total_staked_rao: u64,
}

/// Handle a GetStakeInfo request.
///
/// Phase 1 stub: Returns empty list since staking is not active.
pub async fn handle_get_stake_info(
    _request: GetStakeInfoRequest,
) -> Result<GetStakeInfoResponse, String> {
    // Phase 3: Query chitin_economics::StakeManager for stake data
    Ok(GetStakeInfoResponse {
        stakes: Vec::new(),
        total_staked_rao: 0,
    })
}
