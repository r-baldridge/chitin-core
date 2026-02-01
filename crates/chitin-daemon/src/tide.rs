// crates/chitin-daemon/src/tide.rs
//
// TideNode: Validation and scoring pipeline for the Chitin Protocol.
//
// Tide Nodes evaluate Polyp quality via multi-dimensional scoring,
// participate in Yuma-Semantic Consensus, and submit weight vectors.
//
// Phase 4: Epoch-event-driven validation pipeline. On Scoring phase,
// scores polyps and populates weight matrix. On EpochBoundary, triggers
// consensus runner.

use std::sync::Arc;

use tokio::sync::broadcast;

use chitin_consensus::epoch::EpochPhase;
use chitin_consensus::scoring::score_polyp_multi_dimensional;
use chitin_core::traits::PolypStore;
use chitin_core::PolypState;
use chitin_store::RocksStore;

use crate::config::DaemonConfig;
use crate::consensus_runner;
use crate::epoch_events::EpochEvent;
use crate::shared::DaemonSharedState;

/// A Tide Node that validates and scores Polyps.
pub struct TideNode {
    #[allow(dead_code)]
    config: DaemonConfig,
    /// Broadcast receiver for epoch events.
    event_rx: broadcast::Receiver<EpochEvent>,
    /// Shared daemon state.
    shared: DaemonSharedState,
    /// Polyp store for reading polyps to score.
    store: Arc<RocksStore>,
}

impl TideNode {
    /// Create a new TideNode with the given configuration and shared state.
    pub fn new(
        config: &DaemonConfig,
        event_rx: broadcast::Receiver<EpochEvent>,
        shared: DaemonSharedState,
        store: Arc<RocksStore>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            config: config.clone(),
            event_rx,
            shared,
            store,
        })
    }

    /// Start the Tide Node event loop.
    ///
    /// Listens for epoch events and runs validation/scoring pipelines.
    pub async fn start(mut self) -> Result<(), Box<dyn std::error::Error>> {
        tracing::info!("Tide node started (epoch-event-driven)");

        loop {
            tokio::select! {
                _ = tokio::signal::ctrl_c() => {
                    tracing::info!("Tide node received shutdown signal");
                    break;
                }
                event = self.event_rx.recv() => {
                    match event {
                        Ok(EpochEvent::PhaseChanged { epoch, phase, block }) => {
                            self.handle_phase_change(epoch, phase, block).await;
                        }
                        Ok(EpochEvent::EpochBoundary { epoch, block }) => {
                            self.handle_epoch_boundary(epoch, block).await;
                        }
                        Err(broadcast::error::RecvError::Lagged(n)) => {
                            tracing::warn!("Tide node lagged behind {} epoch events", n);
                        }
                        Err(broadcast::error::RecvError::Closed) => {
                            tracing::info!("Epoch event channel closed, shutting down");
                            break;
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Handle a phase change event.
    async fn handle_phase_change(&self, epoch: u64, phase: EpochPhase, _block: u64) {
        if phase == EpochPhase::Scoring {
            tracing::info!("Epoch {}: Scoring phase — running validation pipeline", epoch);
            if let Err(e) = self.run_scoring_pipeline(epoch).await {
                tracing::error!("Scoring pipeline failed: {}", e);
            }
        }
    }

    /// Handle an epoch boundary event.
    async fn handle_epoch_boundary(&self, epoch: u64, _block: u64) {
        tracing::info!("Epoch {}: Boundary — triggering consensus", epoch);
        if let Err(e) = consensus_runner::run_epoch_consensus(&self.shared, &self.store, epoch).await {
            tracing::error!("Consensus runner failed at epoch {}: {}", epoch, e);
        }
    }

    /// Score all Soft and UnderReview polyps, populate the weight matrix.
    async fn run_scoring_pipeline(&self, epoch: u64) -> Result<(), String> {
        // List Soft and UnderReview polyps
        let soft_polyps = self.store.list_polyps_by_state(&PolypState::Soft).await
            .map_err(|e| format!("Failed to list Soft polyps: {}", e))?;
        let under_review_polyps = self.store.list_polyps_by_state(&PolypState::UnderReview).await
            .map_err(|e| format!("Failed to list UnderReview polyps: {}", e))?;

        let mut all_polyps = soft_polyps;
        all_polyps.extend(under_review_polyps);

        if all_polyps.is_empty() {
            tracing::info!("Epoch {}: No polyps to score", epoch);
            return Ok(());
        }

        tracing::info!("Epoch {}: Scoring {} polyps", epoch, all_polyps.len());

        // Score each polyp and collect weighted scores grouped by creator hotkey
        // For Phase 4, we operate as a single validator (uid=0)
        // and assign coral indices sequentially based on polyp ordering.
        let n_corals = all_polyps.len();

        // Resize weight matrix: 1 validator, n_corals coral nodes
        {
            let mut wm = self.shared.weight_matrix.write().await;
            *wm = chitin_consensus::weights::WeightMatrix::new(1, n_corals);

            for (coral_idx, polyp) in all_polyps.iter().enumerate() {
                let scores = score_polyp_multi_dimensional(polyp);
                let weight = scores.weighted_score();
                wm.set(0, coral_idx, weight);
            }

            wm.normalize();
        }

        // Transition Soft polyps to UnderReview
        for polyp in &all_polyps {
            if polyp.state == PolypState::Soft {
                let mut updated = polyp.clone();
                updated.state = PolypState::UnderReview;
                updated.updated_at = chrono::Utc::now();
                if let Err(e) = self.store.save_polyp(&updated).await {
                    tracing::warn!("Failed to transition polyp {} to UnderReview: {}", polyp.id, e);
                }
            }
        }

        tracing::info!(
            "Epoch {}: Scored {} polyps, weight matrix populated",
            epoch,
            all_polyps.len()
        );

        Ok(())
    }
}
