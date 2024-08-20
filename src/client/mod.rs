//! # Client
//!
//! Client crate mocks the `Client` struct in `bitcoincore-rpc`.

use crate::{ledger::Ledger, utils};
use bitcoin::Txid;
use bitcoincore_rpc::{Auth, RpcApi};

mod rpc_api;

/// This trait defines non-functional interfaces for RPC interfaces, like
/// `new()`. This is needed if target application wants to choose actual rpc and
/// this via trait definitions. This is helpful for choosing different rpc
/// interface between test and release builds.
pub trait RpcApiWrapper: RpcApi + std::marker::Sync + std::marker::Send + 'static {
    /// Create a new RPC connection.
    fn new(url: &str, auth: Auth) -> bitcoincore_rpc::Result<Self>;
    /// Create a new RPC connection, without any cleanup. Useful for `clone`
    /// calls.
    fn new_without_cleanup(url: &str, auth: Auth) -> bitcoincore_rpc::Result<Self>;
}

/// Compatibility implementation for `bitcoincore_rpc::Client`.
impl RpcApiWrapper for bitcoincore_rpc::Client {
    fn new(url: &str, auth: Auth) -> bitcoincore_rpc::Result<Self> {
        bitcoincore_rpc::Client::new(url, auth)
    }

    /// Same as `new`.
    fn new_without_cleanup(url: &str, auth: Auth) -> bitcoincore_rpc::Result<Self> {
        bitcoincore_rpc::Client::new(url, auth)
    }
}

/// Mock Bitcoin RPC client.
#[derive(Clone)]
pub struct Client {
    /// Bitcoin ledger.
    ledger: Ledger,
}

impl RpcApiWrapper for Client {
    /// Creates a new mock Client interface.
    ///
    /// # Parameters
    ///
    /// Parameters must match `bitcoincore_rpc::Client::new()`. Only the `url`
    /// is used for database identification. Authorization struct is not used
    /// and can be a dummy value.
    #[tracing::instrument]
    fn new(url: &str, _auth: bitcoincore_rpc::Auth) -> bitcoincore_rpc::Result<Self> {
        if let Err(e) = utils::initialize_logger() {
            return Err(bitcoincore_rpc::Error::ReturnedError(e.to_string()));
        };

        Ok(Self {
            ledger: Ledger::new(url),
        })
    }

    /// This function will establish a new database connection, while preserving
    /// it's previous state. Meaning it won't clear database. This is helpful
    /// for cloning the `Client` structure.
    fn new_without_cleanup(url: &str, _auth: Auth) -> bitcoincore_rpc::Result<Self> {
        Ok(Self {
            ledger: Ledger::new_without_cleanup(url),
        })
    }
}

/// Dumps complete ledger to a string and returns it. This can help identify
/// bugs as it draws the big picture of the mock blockchain.
pub fn dump_ledger(rpc: Client, pretty: bool) -> String {
    dump_ledger_inner(rpc.ledger, pretty)
}
/// Parent of `dump_ledger`. This function accepts private `Ledger` struct. This
/// useful for only crate tests.
pub fn dump_ledger_inner(ledger: Ledger, pretty: bool) -> String {
    let mut dump = String::new();

    const DELIMETER: &str = "\n-----\n";

    let transactions = ledger.get_transactions();

    if pretty {
        dump += DELIMETER;
        dump += format!("Transactions: {:#?}", transactions).as_str();
        dump += DELIMETER;
        dump += format!(
            "Txids: {:#?}",
            transactions
                .iter()
                .map(|tx| tx.compute_txid())
                .collect::<Vec<Txid>>()
        )
        .as_str();
        dump += DELIMETER;
    } else {
        dump += format!("Transactions: {:?}", transactions).as_str();
        dump += DELIMETER;
        dump += format!(
            "Txids: {:?}",
            transactions
                .iter()
                .map(|tx| tx.compute_txid())
                .collect::<Vec<Txid>>()
        )
        .as_str();
        dump += DELIMETER;
    }

    dump
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Creating a new `Client` with dummy parameters should not panic.
    #[test]
    fn new() {
        let _should_not_panic = Client::new("client_new", bitcoincore_rpc::Auth::None).unwrap();
    }
}
