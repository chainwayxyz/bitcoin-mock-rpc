//! # Bitcoin Ledger
//!
//! Mock Bitcoin ledger.
//!
//! This crate is designed to be used as immutable, because of the `RpcApi`'s
//! immutable nature.

use bitcoin::{Address, TxOut};
use std::cell::Cell;

mod transactions;

/// Mock Bitcoin ledger.
#[derive(Default)]
pub struct Ledger {
    /// User's addresses.
    addresses: Cell<Vec<Address>>,
    /// User's unspent transaction outputs.
    utxos: Cell<Vec<TxOut>>,
}

impl Ledger {
    /// Creates a new empty ledger.
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    /// Adds a new address for user.
    pub fn add_address(&self, address: Address) {
        let mut addresses = self.addresses.take();
        addresses.push(address);

        self.addresses.set(addresses);
    }
}

#[cfg(test)]
mod tests {
    use crate::test_common;

    use super::*;

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
