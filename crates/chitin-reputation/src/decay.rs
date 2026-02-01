// crates/chitin-reputation/src/decay.rs
//
// Time-decay functions for trust scores in the Chitin Protocol.
//
// Trust scores decay over time to ensure nodes must continue participating
// to maintain their reputation. Supports exponential and linear decay.

use serde::{Deserialize, Serialize};

/// Decay function for trust score attenuation over time.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DecayFunction {
    /// Exponential decay: value * 0.5^(elapsed / half_life).
    /// Trust halves every `half_life_epochs` epochs.
    Exponential {
        /// Number of epochs for the trust value to halve.
        half_life_epochs: u64,
    },
    /// Linear decay: max(0, value - decay_per_epoch * elapsed).
    /// Trust decreases by a fixed amount each epoch until reaching zero.
    Linear {
        /// Amount of trust lost per epoch.
        decay_per_epoch: f64,
    },
}

/// Apply a decay function to a trust value.
///
/// # Arguments
/// * `value` - The original trust value.
/// * `epochs_elapsed` - Number of epochs since the value was last updated.
/// * `function` - The decay function to apply.
///
/// # Returns
/// The decayed trust value, always >= 0.0.
pub fn apply_decay(value: f64, epochs_elapsed: u64, function: &DecayFunction) -> f64 {
    match function {
        DecayFunction::Exponential { half_life_epochs } => {
            if *half_life_epochs == 0 {
                return 0.0;
            }
            // Exponential decay: value * 0.5^(elapsed / half_life)
            let exponent = epochs_elapsed as f64 / *half_life_epochs as f64;
            value * (0.5_f64).powf(exponent)
        }
        DecayFunction::Linear { decay_per_epoch } => {
            // Linear decay: max(0, value - decay_per_epoch * elapsed)
            let decayed = value - decay_per_epoch * epochs_elapsed as f64;
            decayed.max(0.0)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exponential_decay_half_life() {
        let func = DecayFunction::Exponential { half_life_epochs: 10 };
        let result = apply_decay(1.0, 10, &func);
        assert!((result - 0.5).abs() < 1e-10);
    }

    #[test]
    fn test_exponential_decay_zero() {
        let func = DecayFunction::Exponential { half_life_epochs: 10 };
        let result = apply_decay(1.0, 0, &func);
        assert!((result - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_linear_decay() {
        let func = DecayFunction::Linear { decay_per_epoch: 0.1 };
        let result = apply_decay(1.0, 5, &func);
        assert!((result - 0.5).abs() < 1e-10);
    }

    #[test]
    fn test_linear_decay_floors_at_zero() {
        let func = DecayFunction::Linear { decay_per_epoch: 0.1 };
        let result = apply_decay(1.0, 20, &func);
        assert!((result - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_exponential_decay_zero_half_life() {
        let func = DecayFunction::Exponential { half_life_epochs: 0 };
        let result = apply_decay(1.0, 5, &func);
        assert!((result - 0.0).abs() < 1e-10);
    }
}
