// crates/chitin-daemon/src/scheduler.rs
//
// Epoch scheduler for the Chitin Protocol daemon.
//
// Simulates block progression with configurable intervals, updates the
// shared EpochManager, detects phase transitions, and broadcasts EpochEvents
// to subscribed tasks (TideNode, consensus runner).

use std::sync::Arc;
use std::time::Duration;

use tokio::sync::{broadcast, RwLock};

use chitin_consensus::epoch::{EpochManager, EpochPhase};

use crate::epoch_events::EpochEvent;

/// Scheduler that simulates block progression and triggers epoch transitions.
pub struct EpochScheduler {
    /// Number of blocks in each epoch.
    blocks_per_epoch: u64,
    /// The current block number (0-indexed).
    current_block: u64,
    /// Shared epoch manager for updating epoch state.
    epoch_manager: Arc<RwLock<EpochManager>>,
    /// Broadcast sender for epoch events.
    event_tx: broadcast::Sender<EpochEvent>,
}

impl EpochScheduler {
    /// Create a new EpochScheduler with the given blocks-per-epoch count.
    pub fn new(
        blocks_per_epoch: u64,
        epoch_manager: Arc<RwLock<EpochManager>>,
        event_tx: broadcast::Sender<EpochEvent>,
    ) -> Self {
        Self {
            blocks_per_epoch,
            current_block: 0,
            epoch_manager,
            event_tx,
        }
    }

    /// Run the scheduler loop, advancing blocks at simulated intervals.
    ///
    /// Each block sleeps for ~12 seconds. Updates the EpochManager on each
    /// block, detects phase transitions, and broadcasts events.
    pub async fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        tracing::info!(
            "Epoch scheduler started (blocks_per_epoch={})",
            self.blocks_per_epoch
        );

        loop {
            tokio::select! {
                _ = tokio::signal::ctrl_c() => {
                    tracing::info!("Epoch scheduler received shutdown signal");
                    break;
                }
                _ = tokio::time::sleep(Duration::from_secs(12)) => {
                    self.advance_block().await;
                }
            }
        }

        Ok(())
    }

    /// Advance the block counter by one, update EpochManager, and emit events.
    pub async fn advance_block(&mut self) {
        let prev_phase;
        let prev_epoch;

        // Read previous state
        {
            let em = self.epoch_manager.read().await;
            prev_phase = em.phase().clone();
            prev_epoch = em.current_epoch();
        }

        self.current_block += 1;

        // Update epoch manager with new block
        {
            let mut em = self.epoch_manager.write().await;
            em.advance_block(self.current_block);
        }

        let new_phase;
        let new_epoch;

        // Read new state
        {
            let em = self.epoch_manager.read().await;
            new_phase = em.phase().clone();
            new_epoch = em.current_epoch();
        }

        // Detect epoch boundary
        if new_epoch > prev_epoch {
            tracing::info!(
                "=== EPOCH {} BOUNDARY === (block {})",
                new_epoch,
                self.current_block
            );
            let _ = self.event_tx.send(EpochEvent::EpochBoundary {
                epoch: new_epoch,
                block: self.current_block,
            });
        }

        // Detect phase transition
        if new_phase != prev_phase {
            tracing::info!(
                "Phase transition: {:?} -> {:?} (epoch {}, block {})",
                prev_phase,
                new_phase,
                new_epoch,
                self.current_block
            );
            let _ = self.event_tx.send(EpochEvent::PhaseChanged {
                epoch: new_epoch,
                phase: new_phase,
                block: self.current_block,
            });
        } else {
            let block_in_epoch = self.current_block % self.blocks_per_epoch;
            tracing::trace!(
                "Block {} (epoch {}, block {}/{})",
                self.current_block,
                new_epoch,
                block_in_epoch,
                self.blocks_per_epoch
            );
        }
    }
}
