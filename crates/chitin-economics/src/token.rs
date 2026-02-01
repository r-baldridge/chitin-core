// crates/chitin-economics/src/token.rs
//
// $CTN (Chitin) token type and supply constants.
//
// The smallest unit of $CTN is the "rao" (named after the Bittensor unit).
// 1 CTN = 10^9 rao. All internal accounting uses rao to avoid floating-point
// precision issues in economic calculations.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::ops::{Add, Sub};

/// Number of rao in one CTN. 1 CTN = 10^9 rao.
pub const RAO_PER_CTN: u64 = 1_000_000_000;

/// Maximum supply of $CTN in rao. 21,000,000 CTN * 10^9 rao/CTN.
pub const MAX_SUPPLY_RAO: u64 = 21_000_000 * RAO_PER_CTN;

/// Type alias for rao — the smallest unit of $CTN.
pub type Rao = u64;

/// The $CTN (Chitin) token amount.
///
/// Wraps an amount in rao (the smallest denomination).
/// All arithmetic is performed in integer rao to avoid floating-point errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Ctn {
    /// Amount in rao (1 CTN = 10^9 rao).
    pub rao: u64,
}

impl Ctn {
    /// Create a Ctn amount from a whole CTN value (as f64).
    ///
    /// # Example
    /// ```
    /// use chitin_economics::token::Ctn;
    /// let amount = Ctn::from_ctn(1.5);
    /// assert_eq!(amount.rao, 1_500_000_000);
    /// ```
    pub fn from_ctn(amount: f64) -> Self {
        Self {
            rao: (amount * RAO_PER_CTN as f64) as u64,
        }
    }

    /// Create a Ctn amount from a rao value.
    pub fn from_rao(rao: u64) -> Self {
        Self { rao }
    }

    /// Convert this amount to CTN as a floating-point value.
    pub fn to_ctn(&self) -> f64 {
        self.rao as f64 / RAO_PER_CTN as f64
    }

    /// Returns zero CTN.
    pub fn zero() -> Self {
        Self { rao: 0 }
    }
}

impl Add for Ctn {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            rao: self.rao + rhs.rao,
        }
    }
}

impl Sub for Ctn {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            rao: self.rao.saturating_sub(rhs.rao),
        }
    }
}

impl fmt::Display for Ctn {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let whole = self.rao / RAO_PER_CTN;
        let frac = self.rao % RAO_PER_CTN;
        if frac == 0 {
            write!(f, "{} CTN", whole)
        } else {
            // Display up to 9 decimal places, trimming trailing zeros
            let frac_str = format!("{:09}", frac);
            let trimmed = frac_str.trim_end_matches('0');
            write!(f, "{}.{} CTN", whole, trimmed)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rao_per_ctn() {
        assert_eq!(RAO_PER_CTN, 1_000_000_000);
    }

    #[test]
    fn test_max_supply() {
        // 21_000_000 * 1_000_000_000 = 21_000_000_000_000_000
        assert_eq!(MAX_SUPPLY_RAO, 21_000_000 * RAO_PER_CTN);
    }

    #[test]
    fn test_from_ctn() {
        let amount = Ctn::from_ctn(1.0);
        assert_eq!(amount.rao, RAO_PER_CTN);

        let amount = Ctn::from_ctn(0.5);
        assert_eq!(amount.rao, 500_000_000);
    }

    #[test]
    fn test_to_ctn() {
        let amount = Ctn::from_rao(RAO_PER_CTN);
        assert!((amount.to_ctn() - 1.0).abs() < f64::EPSILON);

        let amount = Ctn::from_rao(1_500_000_000);
        assert!((amount.to_ctn() - 1.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_add() {
        let a = Ctn::from_ctn(1.0);
        let b = Ctn::from_ctn(2.5);
        let c = a + b;
        assert_eq!(c.rao, 3_500_000_000);
    }

    #[test]
    fn test_sub() {
        let a = Ctn::from_ctn(3.0);
        let b = Ctn::from_ctn(1.5);
        let c = a - b;
        assert_eq!(c.rao, 1_500_000_000);
    }

    #[test]
    fn test_sub_saturating() {
        let a = Ctn::from_ctn(1.0);
        let b = Ctn::from_ctn(2.0);
        let c = a - b;
        assert_eq!(c.rao, 0); // saturating subtraction
    }

    #[test]
    fn test_display_whole() {
        let amount = Ctn::from_ctn(42.0);
        assert_eq!(format!("{}", amount), "42 CTN");
    }

    #[test]
    fn test_display_fractional() {
        let amount = Ctn::from_rao(1_500_000_000);
        assert_eq!(format!("{}", amount), "1.5 CTN");
    }

    #[test]
    fn test_display_zero() {
        let amount = Ctn::zero();
        assert_eq!(format!("{}", amount), "0 CTN");
    }
}
