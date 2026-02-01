// crates/chitin-daemon/src/consensus_runner.rs
//
// Epoch boundary consensus execution for the Chitin Protocol daemon.
//
// Called by TideNode at each EpochBoundary event. Reads the weight and bond
// matrices from shared state, runs Yuma-Semantic Consensus, stores the result,
// updates bonds, identifies approved polyps, and triggers hardening.

use std::sync::Arc;

use chitin_consensus::yuma::yuma_semantic_consensus;
use chitin_core::consensus::ConsensusMetadata;
use chitin_core::traits::PolypStore;
use chitin_core::PolypState;
use chitin_store::RocksStore;

use crate::hardening_pipeline;
use crate::shared::DaemonSharedState;

/// Consensus weight threshold: polyps with consensus_weight above this are approved.
const APPROVAL_THRESHOLD: f64 = 0.3;

/// Run epoch consensus at an epoch boundary.
///
/// Steps:
/// 1. Read weight and bond matrices from shared state
/// 2. Gather stakes (Phase 4: equal stake=100 for all validators)
/// 3. Run yuma_semantic_consensus
/// 4. Store ConsensusResult in shared state
/// 5. Update bond matrix with result bonds
/// 6. Identify approved polyps (consensus_weight > threshold)
/// 7. Transition approved polyps: UnderReview -> Approved
/// 8. Trigger hardening pipeline for approved polyps
/// 9. Update trust matrix from validator agreement
/// 10. Update metagraph with new epoch state
pub async fn run_epoch_consensus(
    shared: &DaemonSharedState,
    store: &Arc<RocksStore>,
    epoch: u64,
) -> Result<(), String> {
    // Step 1: Read weight and bond matrices
    let weights;
    let prev_bonds;
    let n_validators;
    let n_corals;

    {
        let wm = shared.weight_matrix.read().await;
        weights = wm.weights.clone();
        n_validators = weights.len();
        n_corals = if n_validators > 0 { weights[0].len() } else { 0 };
    }

    if n_validators == 0 || n_corals == 0 {
        tracing::info!("Epoch {}: No weights submitted, skipping consensus", epoch);
        return Ok(());
    }

    {
        let bm = shared.bond_matrix.read().await;
        // If bond matrix dimensions don't match, use zeros
        if bm.bonds.len() == n_validators
            && bm.bonds.first().map_or(true, |r| r.len() == n_corals)
        {
            prev_bonds = bm.bonds.clone();
        } else {
            prev_bonds = vec![vec![0.0; n_corals]; n_validators];
        }
    }

    // Step 2: All validators get equal stake=100 in Phase 4
    let stakes: Vec<u64> = vec![100; n_validators];

    tracing::info!(
        "Epoch {}: Running consensus ({} validators, {} corals)",
        epoch,
        n_validators,
        n_corals
    );

    // Step 3: Run Yuma-Semantic Consensus
    let result = yuma_semantic_consensus(&stakes, &weights, &prev_bonds, 0.5, 0.1, 0.1);

    tracing::info!(
        "Epoch {}: Consensus complete — {} consensus weights",
        epoch,
        result.consensus_weights.len()
    );

    // Step 4: Store ConsensusResult in shared state
    {
        let mut cr = shared.last_consensus_result.write().await;
        *cr = Some(result.clone());
    }

    // Step 5: Update bond matrix with result bonds
    {
        let mut bm = shared.bond_matrix.write().await;
        *bm = chitin_consensus::bonds::BondMatrix::new(n_validators, n_corals);
        for (i, row) in result.bonds.iter().enumerate() {
            for (j, &val) in row.iter().enumerate() {
                bm.bonds[i][j] = val;
            }
        }
    }

    // Step 6: Identify approved polyps (consensus_weight > threshold)
    // We need to match consensus weights back to actual polyps.
    // Re-list UnderReview polyps (same order as scored).
    let under_review_polyps = store
        .list_polyps_by_state(&PolypState::UnderReview)
        .await
        .map_err(|e| format!("Failed to list UnderReview polyps: {}", e))?;

    let mut approved_polyps = Vec::new();
    for (idx, polyp) in under_review_polyps.iter().enumerate() {
        if idx < result.consensus_weights.len() && result.consensus_weights[idx] > APPROVAL_THRESHOLD
        {
            approved_polyps.push(polyp.clone());
        }
    }

    tracing::info!(
        "Epoch {}: {} polyps approved (threshold {})",
        epoch,
        approved_polyps.len(),
        APPROVAL_THRESHOLD
    );

    // Step 7: Transition approved polyps: UnderReview -> Approved
    for polyp in &approved_polyps {
        let mut updated = polyp.clone();
        updated.state = PolypState::Approved;
        updated.consensus = Some(ConsensusMetadata {
            epoch,
            final_score: result.consensus_weights
                .get(under_review_polyps.iter().position(|p| p.id == polyp.id).unwrap_or(0))
                .copied()
                .unwrap_or(0.0),
            validator_scores: vec![],
            hardened: false,
            finalized_at: chrono::Utc::now(),
        });
        updated.updated_at = chrono::Utc::now();
        if let Err(e) = store.save_polyp(&updated).await {
            tracing::warn!("Failed to transition polyp {} to Approved: {}", polyp.id, e);
        }
    }

    // Step 8: Trigger hardening pipeline for approved polyps
    if !approved_polyps.is_empty() {
        if let Err(e) = hardening_pipeline::harden_approved_polyps(shared, store, &approved_polyps).await {
            tracing::error!("Hardening pipeline failed: {}", e);
        }
    }

    // Step 9: Update trust matrix from validator agreement
    // For Phase 4 with a single validator, set self-trust to 1.0
    {
        let mut tm = shared.trust_matrix.write().await;
        for v in 0..n_validators {
            tm.set_trust(v as u16, v as u16, 1.0);
        }
    }

    // Step 10: Update metagraph with new epoch state
    {
        let metagraph = chitin_core::ReefMetagraph {
            epoch,
            block: 0, // Phase 4: block tracking is approximate
            nodes: vec![],
            total_stake: stakes.iter().sum(),
            total_hardened_polyps: approved_polyps.len() as u64,
            emission_rate: 0,
            weights: std::collections::HashMap::new(),
            bonds: std::collections::HashMap::new(),
        };

        let mut mm = shared.metagraph_manager.write().await;
        if let Err(e) = mm.update(metagraph) {
            tracing::warn!("Failed to update metagraph: {}", e);
        }
    }

    tracing::info!("Epoch {}: Consensus pipeline complete", epoch);
    Ok(())
}
