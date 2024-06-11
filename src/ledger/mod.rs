//! # Bitcoin Ledger
//!
//! Mock Bitcoin ledger.
//!
//! This crate is designed to be used as immutable, because of the `RpcApi`'s
//! immutable nature.

use bitcoin::{Address, Transaction, TxOut};
use bitcoin_simulator::database::Database;
use std::{
    cell::Cell,
    sync::{Arc, Mutex},
};

mod errors;
mod transactions;

/// Adds a new item to a `Vec` member, guarded by a `Cell`.
#[macro_export]
macro_rules! add_item {
    ($member:expr, $item:expr) => {
        // Update item list.
        let mut items = $member.take();
        items.push($item);

        // Commit new change.
        $member.set(items);
    };
}
/// Returns item `Vec` of a member, guarded by a `Cell`.
#[macro_export]
macro_rules! get_item {
    ($member:expr) => {
        let items = $member.take();
        $member.set(items.clone());

        return items;
    };
}

/// Mock Bitcoin ledger.
pub struct Ledger {
    /// Private database interface. Data will be written to this temporary
    /// database. Note: It is wrapped around an `Arc<Mutex<>>`. This will help
    /// to use this mock in an asynchronous environment, like `async` or threads.
    database: Arc<Mutex<Database>>,
    /// User's addresses.
    addresses: Cell<Vec<Address>>,
    /// User's unspent transaction outputs.
    utxos: Cell<Vec<TxOut>>,
    /// User's transactions.
    transactions: Cell<Vec<Transaction>>,
}

impl Ledger {
    /// Creates a new empty ledger.
    pub fn new() -> Self {
        Self {
            database: Arc::new(Mutex::new(Database::connect_temporary_database().unwrap())),
            addresses: Cell::new(Vec::new()),
            utxos: Cell::new(Vec::new()),
            transactions: Cell::new(Vec::new()),
        }
    }

    /// Adds a new address for the user.
    pub fn add_address(&self, address: Address) {
        add_item!(self.addresses, address);
    }
    /// Returns addresses of the user.
    pub fn _get_addresses(&self) -> Vec<Address> {
        get_item!(self.addresses);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_common;

    #[test]
    fn new() {
        let _should_not_panic = Ledger::new();
    }

    #[test]
    fn add_address() {
        let ledger = Ledger::new();

        assert_eq!(ledger.addresses.take().len(), 0);

        let address = test_common::get_temp_address();
        ledger.add_address(address);
        assert_eq!(ledger.addresses.take().len(), 1);
    }
}
