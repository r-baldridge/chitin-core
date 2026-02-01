// crates/chitin-rpc/src/handlers/validation.rs
//
// Validation and scoring handlers: SubmitScores, GetEpochStatus, GetConsensusResult.
// Phase 4: Wired to live epoch manager and consensus result state.

use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use chitin_consensus::epoch::{EpochManager, EpochPhase};
use chitin_consensus::weights::WeightMatrix;
use chitin_consensus::yuma::ConsensusResult;

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
/// Phase 4: Validates epoch phase is Scoring or Committing, stores weights
/// in the shared weight matrix.
pub async fn handle_submit_scores(
    request: SubmitScoresRequest,
    weight_matrix: Option<&Arc<RwLock<WeightMatrix>>>,
    epoch_manager: Option<&Arc<RwLock<EpochManager>>>,
) -> Result<SubmitScoresResponse, String> {
    // Validate epoch manager is available
    let em = match epoch_manager {
        Some(em) => em,
        None => {
            return Ok(SubmitScoresResponse {
                accepted: false,
                message: "Phase 1 stub: score submission not yet implemented".to_string(),
            });
        }
    };

    // Check epoch phase
    let (current_epoch, phase) = {
        let em = em.read().await;
        (em.current_epoch(), em.phase().clone())
    };

    if request.epoch != current_epoch {
        return Ok(SubmitScoresResponse {
            accepted: false,
            message: format!(
                "Epoch mismatch: submitted for epoch {} but current is {}",
                request.epoch, current_epoch
            ),
        });
    }

    if phase != EpochPhase::Scoring && phase != EpochPhase::Committing {
        return Ok(SubmitScoresResponse {
            accepted: false,
            message: format!(
                "Cannot submit scores during {:?} phase. Wait for Scoring or Committing phase.",
                phase
            ),
        });
    }

    // Store weights in the weight matrix
    if let Some(wm) = weight_matrix {
        let mut wm = wm.write().await;
        // For Phase 4, we use validator_uid=0 (single validator)
        // and store each weight entry by coral_uid
        for entry in &request.weights {
            let coral_idx = entry.coral_uid as usize;
            if coral_idx < wm.weights.get(0).map_or(0, |r| r.len()) {
                wm.set(0, coral_idx, entry.weight);
            }
        }
    }

    Ok(SubmitScoresResponse {
        accepted: true,
        message: format!(
            "Accepted {} weights for epoch {}",
            request.weights.len(),
            request.epoch
        ),
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
/// Phase 4: Reads current epoch and phase from EpochManager.
pub async fn handle_get_epoch_status(
    _request: GetEpochStatusRequest,
    epoch_manager: Option<&Arc<RwLock<EpochManager>>>,
) -> Result<GetEpochStatusResponse, String> {
    match epoch_manager {
        Some(em) => {
            let em = em.read().await;
            let phase_str = match em.phase() {
                EpochPhase::Open => "Open",
                EpochPhase::Scoring => "Scoring",
                EpochPhase::Committing => "Committing",
                EpochPhase::Closed => "Closed",
            };
            Ok(GetEpochStatusResponse {
                epoch: em.current_epoch(),
                phase: phase_str.to_string(),
                blocks_remaining: 0, // Phase 5: compute from block position
                time_remaining_seconds: 0,
                scores_submitted: 0,
                total_validators: 1, // Phase 4: single validator
            })
        }
        None => {
            Ok(GetEpochStatusResponse {
                epoch: 0,
                phase: "Open".to_string(),
                blocks_remaining: 0,
                time_remaining_seconds: 0,
                scores_submitted: 0,
                total_validators: 0,
            })
        }
    }
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
/// Phase 4: Returns the last consensus result from shared state.
pub async fn handle_get_consensus_result(
    _request: GetConsensusResultRequest,
    consensus_result: Option<&Arc<RwLock<Option<ConsensusResult>>>>,
) -> Result<GetConsensusResultResponse, String> {
    match consensus_result {
        Some(cr) => {
            let cr = cr.read().await;
            match cr.as_ref() {
                Some(result) => Ok(GetConsensusResultResponse {
                    finalized: true,
                    consensus_weights: Some(result.consensus_weights.clone()),
                    incentives: Some(result.incentives.clone()),
                    dividends: Some(result.dividends.clone()),
                    hardened_count: result.hardened_polyp_ids.len() as u32,
                }),
                None => Ok(GetConsensusResultResponse {
                    finalized: false,
                    consensus_weights: None,
                    incentives: None,
                    dividends: None,
                    hardened_count: 0,
                }),
            }
        }
        None => Ok(GetConsensusResultResponse {
            finalized: false,
            consensus_weights: None,
            incentives: None,
            dividends: None,
            hardened_count: 0,
        }),
    }
}
