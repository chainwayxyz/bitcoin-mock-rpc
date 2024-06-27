//! # Bitcoin Ledger
//!
//! Mock Bitcoin ledger.
//!
//! This crate is designed to be used as immutable, because of the `RpcApi`'s
//! immutable nature.

use address::UserCredential;
use bitcoin::{OutPoint, Transaction};
use std::{
    cell::Cell,
    sync::{Arc, Mutex},
};

mod address;
mod errors;
mod macros;
mod spending_requirements;
mod transactions;
mod utxo;

/// Mock Bitcoin ledger.
#[derive(Clone)]
pub struct Ledger {
    /// User's keys and address.
    credentials: Box<Arc<Mutex<Cell<Vec<UserCredential>>>>>,
    /// Happened transactions.
    transactions: Box<Arc<Mutex<Cell<Vec<Transaction>>>>>,
    /// Unspent transaction outputs.
    utxos: Box<Arc<Mutex<Cell<Vec<OutPoint>>>>>,
}

impl Ledger {
    /// Creates a new empty ledger.
    pub fn new() -> Self {
        Self {
            credentials: Box::new(Arc::new(Mutex::new(Cell::new(Vec::new())))),
            utxos: Box::new(Arc::new(Mutex::new(Cell::new(Vec::new())))),
            transactions: Box::new(Arc::new(Mutex::new(Cell::new(Vec::new())))),
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
