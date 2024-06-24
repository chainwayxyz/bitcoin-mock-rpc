//! # Bitcoin Ledger
//!
//! Mock Bitcoin ledger.
//!
//! This crate is designed to be used as immutable, because of the `RpcApi`'s
//! immutable nature.

use address::UserCredential;
use bitcoin::{Transaction, TxOut};
use std::cell::Cell;

mod address;
mod errors;
mod macros;
mod transactions;

/// Mock Bitcoin ledger.
pub struct Ledger {
    /// Inner ledger that holds real ledger information.
    _inner: Cell<InnerLedger>,
    /// User's keys and address.
    credentials: Cell<Vec<UserCredential>>,
    /// User's unspent transaction outputs.
    utxos: Cell<Vec<TxOut>>,
    /// User's transactions.
    transactions: Cell<Vec<Transaction>>,
}

/// Real mock ledger that holds history of the transactions and other
/// information. This struct must be wrapped around a synchronization structure,
/// like `Cell`.
struct InnerLedger {}

impl InnerLedger {
    pub fn new() -> Self {
        Self {}
    }
}

impl Ledger {
    /// Creates a new empty ledger.
    ///
    /// # Panics
    ///
    /// If database connection cannot be established in bitcoin-simulator, it
    /// will panic.
    pub fn new() -> Self {
        Self {
            credentials: Cell::new(Vec::new()),
            utxos: Cell::new(Vec::new()),
            transactions: Cell::new(Vec::new()),
            _inner: Cell::new(InnerLedger::new()),
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
