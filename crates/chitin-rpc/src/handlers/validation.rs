// crates/chitin-rpc/src/handlers/validation.rs
//
// Validation and scoring handlers: SubmitScores, GetEpochStatus, GetConsensusResult.
// Phase 1: Stub implementations. Phase 3 will wire into the consensus engine.

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// SubmitScores
// ---------------------------------------------------------------------------

/// A weight entry in the score submission.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeightEntry {
    /// Network UID of the Coral Node being scored.
    pub coral_uid: u16,
    /// The weight (aggregated score) for this Coral Node.
    pub weight: f64,
}

/// Request for a Tide Node to submit epoch scores/weights.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitScoresRequest {
    /// Hex-encoded validator hotkey.
    pub validator_hotkey: String,
    /// Epoch number for which scores are being submitted.
    pub epoch: u64,
    /// Sparse weight vector: (coral_uid, weight) pairs.
    pub weights: Vec<WeightEntry>,
    /// Hex-encoded signature over the score payload.
    pub signature: String,
}

/// Response from score submission.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitScoresResponse {
    /// Whether the submission was accepted.
    pub accepted: bool,
    /// Human-readable message.
    pub message: String,
}

/// Handle a SubmitScores request.
///
/// Phase 1 stub: Score submission is not yet active.
pub async fn handle_submit_scores(
    _request: SubmitScoresRequest,
) -> Result<SubmitScoresResponse, String> {
    // Phase 3: Validate signature, check epoch, store weights
    Ok(SubmitScoresResponse {
        accepted: false,
        message: "Phase 1 stub: score submission not yet implemented".to_string(),
    })
}

// ---------------------------------------------------------------------------
// GetEpochStatus
// ---------------------------------------------------------------------------

/// Request for the current epoch status.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetEpochStatusRequest {}

/// Response containing epoch status.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetEpochStatusResponse {
    /// Current epoch number.
    pub epoch: u64,
    /// Current phase: "Open", "Scoring", "Committing", or "Closed".
    pub phase: String,
    /// Blocks remaining in the current phase.
    pub blocks_remaining: u64,
    /// Estimated time remaining in seconds.
    pub time_remaining_seconds: u64,
    /// Number of validators that have submitted scores this epoch.
    pub scores_submitted: u32,
    /// Total registered validators.
    pub total_validators: u32,
}

/// Handle a GetEpochStatus request.
///
/// Phase 1 stub: Returns epoch 0 in Open phase.
pub async fn handle_get_epoch_status(
    _request: GetEpochStatusRequest,
) -> Result<GetEpochStatusResponse, String> {
    // Phase 2: Read from chitin_consensus::EpochManager
    Ok(GetEpochStatusResponse {
        epoch: 0,
        phase: "Open".to_string(),
        blocks_remaining: 0,
        time_remaining_seconds: 0,
        scores_submitted: 0,
        total_validators: 0,
    })
}

// ---------------------------------------------------------------------------
// GetConsensusResult
// ---------------------------------------------------------------------------

/// Request for the consensus result of a completed epoch.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetConsensusResultRequest {
    /// Epoch number to query.
    pub epoch: u64,
}

/// Response containing the consensus result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetConsensusResultResponse {
    /// Whether the epoch has been finalized.
    pub finalized: bool,
    /// Consensus weights per Coral Node (if finalized).
    pub consensus_weights: Option<Vec<f64>>,
    /// Incentive scores per Coral Node (if finalized).
    pub incentives: Option<Vec<f64>>,
    /// Dividend scores per Tide Node (if finalized).
    pub dividends: Option<Vec<f64>>,
    /// Number of Polyps hardened in this epoch.
    pub hardened_count: u32,
}

/// Handle a GetConsensusResult request.
///
/// Phase 1 stub: No consensus has run yet.
pub async fn handle_get_consensus_result(
    _request: GetConsensusResultRequest,
) -> Result<GetConsensusResultResponse, String> {
    // Phase 3: Look up the ConsensusResult for the requested epoch
    Ok(GetConsensusResultResponse {
        finalized: false,
        consensus_weights: None,
        incentives: None,
        dividends: None,
        hardened_count: 0,
    })
}
