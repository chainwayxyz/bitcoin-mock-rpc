//! # Client
//!
//! Client crate mocks the `Client` struct in `bitcoincore-rpc`.

use crate::ledger::Ledger;
use bitcoin_simulator::database::Database;
use std::{
    cell::Cell,
    sync::{Arc, Mutex},
};

mod rpc_api;

/// Mock Bitcoin RPC client.
pub struct Client {
    /// Private database interface. Data will be written to this temporary
    /// database. Note: It is wrapped around an `Arc<Mutex<>>`. This will help
    /// to use this mock in an asynchronous environment, like `async` or threads.
    database: Arc<Mutex<Database>>,
    /// Bitcoin ledger.
    ledger: Cell<Ledger>,
}

impl Client {
    /// Creates a new mock Client interface.
    ///
    /// # Parameters
    ///
    /// Parameters are just here to match `bitcoincore_rpc::Client::new()`. They
    /// are not used and can be dummy values.
    ///
    /// # Panics
    ///
    /// This function will panic if connection to the SQLite database cannot be
    /// established.
    pub fn new(_url: &str, _auth: bitcoincore_rpc::Auth) -> bitcoincore_rpc::Result<Self> {
        let database = Database::connect_temporary_database().unwrap();

        Ok(Self {
            database: Arc::new(Mutex::new(database)),
            ledger: Cell::new(Ledger::new()),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Creating a new `Client` with dummy parameters should not panic.
    #[test]
    fn new() {
        let _should_not_panic = Client::new("", bitcoincore_rpc::Auth::None).unwrap();
    }
}
