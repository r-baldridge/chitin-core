// crates/chitin-daemon/src/scheduler.rs
//
// Epoch scheduler for the Chitin Protocol daemon.
//
// Simulates block progression with configurable intervals and logs
// epoch transitions at the configured blocks-per-epoch boundary.

use std::time::Duration;

/// Scheduler that simulates block progression and triggers epoch transitions.
pub struct EpochScheduler {
    /// Number of blocks in each epoch.
    blocks_per_epoch: u64,
    /// The current block number (0-indexed).
    current_block: u64,
}

impl EpochScheduler {
    /// Create a new EpochScheduler with the given blocks-per-epoch count.
    pub fn new(blocks_per_epoch: u64) -> Self {
        Self {
            blocks_per_epoch,
            current_block: 0,
        }
    }

    /// Run the scheduler loop, advancing blocks at simulated intervals.
    ///
    /// Each block sleeps for a configurable duration (~12 seconds in production,
    /// reduced here for Phase 1 development). Logs epoch transitions when
    /// the block count crosses an epoch boundary.
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
                    self.advance_block();
                }
            }
        }

        Ok(())
    }

    /// Advance the block counter by one and check for epoch boundary.
    pub fn advance_block(&mut self) {
        self.current_block += 1;
        let epoch = self.current_block / self.blocks_per_epoch;
        let block_in_epoch = self.current_block % self.blocks_per_epoch;

        if block_in_epoch == 0 {
            tracing::info!(
                "=== EPOCH {} BOUNDARY === (block {})",
                epoch,
                self.current_block
            );
        } else {
            tracing::trace!(
                "Block {} (epoch {}, block {}/{})",
                self.current_block,
                epoch,
                block_in_epoch,
                self.blocks_per_epoch
            );
        }
    }
}
