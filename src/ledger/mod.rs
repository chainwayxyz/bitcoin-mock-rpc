//! # Bitcoin Ledger
//!
//! Mock Bitcoin ledger.
//!
//! This crate is designed to be used as immutable, because of the `RpcApi`'s
//! immutable nature.

use address::UserCredential;
use bitcoin::{OutPoint, Transaction};
use std::cell::Cell;

mod address;
mod errors;
mod macros;
mod transactions;

/// Mock Bitcoin ledger.
pub struct Ledger {
    /// User's keys and address.
    credentials: Cell<Vec<UserCredential>>,
    /// Happened transactions.
    transactions: Cell<Vec<Transaction>>,
    /// Unspent transaction outputs.
    utxos: Cell<Vec<OutPoint>>,
}

impl Ledger {
    /// Creates a new empty ledger.
    pub fn new() -> Self {
        Self {
            credentials: Cell::new(Vec::new()),
            utxos: Cell::new(Vec::new()),
            transactions: Cell::new(Vec::new()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new() {
        let _should_not_panic = Ledger::new();
    }
}
