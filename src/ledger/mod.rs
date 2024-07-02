//! # Bitcoin Ledger
//!
//! Mock Bitcoin ledger.
//!
//! This crate is designed to be used as immutable, because of the `RpcApi`'s
//! immutable nature.

use address::UserCredential;
use bitcoin::{Address, OutPoint, Transaction};
use std::{
    cell::Cell,
    collections::HashMap,
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
    /// Unspent transaction outputs, for every addresses.
    utxos: Box<Arc<Mutex<Cell<HashMap<Address, Vec<OutPoint>>>>>>,
}

impl Ledger {
    /// Creates a new empty ledger.
    pub fn new() -> Self {
        Self {
            credentials: Box::new(Arc::new(Mutex::new(Cell::new(Vec::new())))),
            transactions: Box::new(Arc::new(Mutex::new(Cell::new(Vec::new())))),
            utxos: Box::new(Arc::new(Mutex::new(Cell::new(HashMap::new())))),
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
