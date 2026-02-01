// crates/chitin-daemon/src/hardening_pipeline.rs
//
// Post-consensus hardening pipeline for the Chitin Protocol daemon.
//
// After consensus identifies approved polyps, this module serializes them
// to IPFS via HardenedStore, generates Merkle proofs via HardeningManager,
// and updates polyp state to Hardened.

use std::sync::Arc;

use chitin_consensus::hardening::HardeningManager;
use chitin_core::polyp::Polyp;
use chitin_core::traits::PolypStore;
use chitin_core::PolypState;
use chitin_store::RocksStore;

use crate::shared::DaemonSharedState;

/// Harden all approved polyps through IPFS storage and Merkle proof generation.
///
/// For each approved polyp:
/// 1. Serialize to IPFS via HardenedStore::store_hardened()
/// 2. Pin + generate Merkle proof via HardeningManager::harden_polyp()
/// 3. Update polyp state to Hardened with hardening lineage
/// 4. Save updated polyp back to store
pub async fn harden_approved_polyps(
    shared: &DaemonSharedState,
    store: &Arc<RocksStore>,
    approved_polyps: &[Polyp],
) -> Result<(), String> {
    let hardened_store = match &shared.hardened_store {
        Some(hs) => hs.clone(),
        None => {
            tracing::warn!("No hardened store configured, skipping hardening pipeline");
            return Ok(());
        }
    };

    tracing::info!("Hardening {} approved polyps", approved_polyps.len());

    let mut hardened_count = 0;

    for polyp in approved_polyps {
        match harden_single_polyp(&hardened_store, store, polyp).await {
            Ok(()) => {
                hardened_count += 1;
                tracing::debug!("Hardened polyp {}", polyp.id);
            }
            Err(e) => {
                tracing::error!("Failed to harden polyp {}: {}", polyp.id, e);
            }
        }
    }

    tracing::info!(
        "Hardening complete: {}/{} polyps hardened",
        hardened_count,
        approved_polyps.len()
    );

    Ok(())
}

/// Harden a single polyp: store to IPFS, pin, generate Merkle proof, update state.
async fn harden_single_polyp(
    hardened_store: &Arc<chitin_store::HardenedStore>,
    store: &Arc<RocksStore>,
    polyp: &Polyp,
) -> Result<(), String> {
    // Step 1: Serialize to IPFS via HardenedStore
    let cid = hardened_store
        .store_hardened(polyp)
        .await
        .map_err(|e| format!("Failed to store hardened polyp: {}", e))?;

    // Step 2: Pin + Merkle proof via HardeningManager
    let manager = HardeningManager::new(hardened_store.ipfs.clone());
    let lineage = manager
        .harden_polyp(polyp.id, cid)
        .await
        .map_err(|e| format!("Failed to harden polyp: {}", e))?;

    // Step 3: Update polyp state to Hardened with lineage
    let mut updated = polyp.clone();
    updated.state = PolypState::Hardened;
    updated.hardening = Some(lineage);
    // Mark consensus metadata as hardened if present
    if let Some(ref mut consensus) = updated.consensus {
        consensus.hardened = true;
    }
    updated.updated_at = chrono::Utc::now();

    // Step 4: Save back to store
    store
        .save_polyp(&updated)
        .await
        .map_err(|e| format!("Failed to save hardened polyp: {}", e))?;

    Ok(())
}
