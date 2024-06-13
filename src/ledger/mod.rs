//! # Bitcoin Ledger
//!
//! Mock Bitcoin ledger.
//!
//! This crate is designed to be used as immutable, because of the `RpcApi`'s
//! immutable nature.

use address::UserCredential;
use bitcoin::{Transaction, TxOut};
use bitcoin_simulator::database::Database;
use std::{
    cell::Cell,
    sync::{Arc, Mutex},
};

mod address;
mod errors;
mod macros;
mod transactions;

/// Mock Bitcoin ledger.
pub struct Ledger {
    /// Private database interface. Data will be written to this temporary
    /// database. Note: It is wrapped around an `Arc<Mutex<>>`. This will help
    /// to use this mock in an asynchronous environment, like `async` or threads.
    database: Arc<Mutex<Database>>,
    /// User's keys and address.
    credentials: Cell<Vec<UserCredential>>,
    /// User's unspent transaction outputs.
    utxos: Cell<Vec<TxOut>>,
    /// User's transactions.
    transactions: Cell<Vec<Transaction>>,
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
            database: Arc::new(Mutex::new(Database::connect_temporary_database().unwrap())),
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
