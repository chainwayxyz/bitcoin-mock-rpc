//! # Bitcoin Ledger
//!
//! Mock Bitcoin ledger.
//!
//! This crate is designed to be used as immutable, because of the `RpcApi`'s
//! immutable nature.

use bitcoin::{Address, TxOut};

mod transactions;

/// Mock Bitcoin ledger.
#[derive(Clone, Default)]
pub struct Ledger {
    /// User's addresses.
    pub addresses: Vec<Address>,
    /// User's unspent transaction outputs.
    pub utxos: Vec<TxOut>,
}

impl Ledger {
    /// Creates a new empty ledger.
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    /// Adds a new address for user.
    pub fn add_address(&self, address: Address) -> Self {
        let mut ledger = self.clone().to_owned();

        ledger.addresses.push(address);

        ledger
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
