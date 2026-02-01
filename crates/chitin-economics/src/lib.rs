// crates/chitin-economics/src/lib.rs
//
// chitin-economics: $CTN token economics, emission, staking, rewards,
// slashing, and treasury management for the Chitin Protocol.
//
// All monetary values are tracked in rao (the smallest unit of $CTN).
// 1 CTN = 1,000,000,000 rao (10^9).

pub mod emission;
pub mod rewards;
pub mod slashing;
pub mod staking;
pub mod token;
pub mod treasury;

// Re-export key types for ergonomic access from downstream crates.
pub use emission::{
    cumulative_emission, emission_at_block, epoch_emission, HALVING_INTERVAL,
    INITIAL_BLOCK_REWARD_RAO, TREASURY_FRACTION, VALIDATOR_FRACTION,
};
pub use rewards::{compute_rewards, RewardDistribution};
pub use slashing::{compute_penalty, SlashCondition, SlashResult};
pub use staking::{StakeEntry, StakeManager};
pub use token::{Ctn, Rao, MAX_SUPPLY_RAO, RAO_PER_CTN};
pub use treasury::Treasury;
