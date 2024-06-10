//! # Client
//!
//! Client crate mocks the `Client` struct in `bitcoincore-rpc`.

use crate::ledger::Ledger;
use bitcoin_simulator::database::Database;

mod rpc_api;

/// Mock Bitcoin RPC client.
pub struct Client {
    /// Bitcoin ledger.
    ledger: Ledger,
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
            ledger: Ledger::new(),
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
