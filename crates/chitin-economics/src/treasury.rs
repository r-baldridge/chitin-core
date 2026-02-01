// crates/chitin-economics/src/treasury.rs
//
// Protocol treasury management for the Chitin Protocol.
//
// The treasury receives:
//   - 2% of each epoch's emission (TREASURY_FRACTION)
//   - All slashed tokens from penalty execution
//
// Treasury spending is governed by $CTN token-weighted voting (Phase 3+).
// Uses include: development funding, security audits, migration incentives
// (molting), and ecosystem grants.
//
// Reference: ARCHITECTURE.md Section 7.5

use chitin_core::error::ChitinError;

/// The protocol treasury.
///
/// Tracks the total balance of $CTN held in the treasury (in rao).
/// Deposits come from emission allocation and slashing proceeds.
/// Withdrawals are governed by governance (Phase 3+).
pub struct Treasury {
    /// Current balance in rao.
    balance: u64,
}

impl Treasury {
    /// Create a new treasury with zero balance.
    pub fn new() -> Self {
        Self { balance: 0 }
    }

    /// Create a treasury with an initial balance (in rao).
    pub fn with_balance(balance: u64) -> Self {
        Self { balance }
    }

    /// Deposit tokens into the treasury.
    ///
    /// # Arguments
    /// - `amount` — Amount to deposit in rao.
    pub fn deposit(&mut self, amount: u64) {
        self.balance = self.balance.saturating_add(amount);
    }

    /// Withdraw tokens from the treasury.
    ///
    /// # Arguments
    /// - `amount` — Amount to withdraw in rao.
    ///
    /// # Errors
    /// Returns `ChitinError::InvalidState` if the treasury has insufficient balance.
    pub fn withdraw(&mut self, amount: u64) -> Result<(), ChitinError> {
        if amount > self.balance {
            return Err(ChitinError::InvalidState(format!(
                "Insufficient treasury balance: requested {} rao but only {} rao available",
                amount, self.balance
            )));
        }
        self.balance -= amount;
        Ok(())
    }

    /// Get the current treasury balance (in rao).
    pub fn balance(&self) -> u64 {
        self.balance
    }
}

impl Default for Treasury {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::token::RAO_PER_CTN;

    #[test]
    fn test_new_treasury_has_zero_balance() {
        let treasury = Treasury::new();
        assert_eq!(treasury.balance(), 0);
    }

    #[test]
    fn test_deposit() {
        let mut treasury = Treasury::new();
        treasury.deposit(100 * RAO_PER_CTN);
        assert_eq!(treasury.balance(), 100 * RAO_PER_CTN);
    }

    #[test]
    fn test_multiple_deposits() {
        let mut treasury = Treasury::new();
        treasury.deposit(50 * RAO_PER_CTN);
        treasury.deposit(30 * RAO_PER_CTN);
        assert_eq!(treasury.balance(), 80 * RAO_PER_CTN);
    }

    #[test]
    fn test_withdraw_success() {
        let mut treasury = Treasury::with_balance(100 * RAO_PER_CTN);
        assert!(treasury.withdraw(40 * RAO_PER_CTN).is_ok());
        assert_eq!(treasury.balance(), 60 * RAO_PER_CTN);
    }

    #[test]
    fn test_withdraw_exact_balance() {
        let mut treasury = Treasury::with_balance(100 * RAO_PER_CTN);
        assert!(treasury.withdraw(100 * RAO_PER_CTN).is_ok());
        assert_eq!(treasury.balance(), 0);
    }

    #[test]
    fn test_withdraw_insufficient_balance() {
        let mut treasury = Treasury::with_balance(50 * RAO_PER_CTN);
        let result = treasury.withdraw(100 * RAO_PER_CTN);
        assert!(result.is_err());
        // Balance should be unchanged
        assert_eq!(treasury.balance(), 50 * RAO_PER_CTN);
    }

    #[test]
    fn test_deposit_after_withdraw() {
        let mut treasury = Treasury::with_balance(100 * RAO_PER_CTN);
        treasury.withdraw(60 * RAO_PER_CTN).unwrap();
        treasury.deposit(20 * RAO_PER_CTN);
        assert_eq!(treasury.balance(), 60 * RAO_PER_CTN);
    }
}
