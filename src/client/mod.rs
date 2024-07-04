//! # Client
//!
//! Client crate mocks the `Client` struct in `bitcoincore-rpc`.

use crate::ledger::Ledger;
use bitcoin::Txid;
use bitcoincore_rpc::{Auth, RpcApi};

mod rpc_api;

/// This trait defines non-functional interfaces for RPC interfaces, like
/// `new()`. This is needed if target application wants to choose actual rpc and
/// this via trait definitions. This is helpful for choosing different rpc
/// interface between test and release builds.
pub trait RpcApiWrapper: RpcApi + std::marker::Sync + std::marker::Send + 'static {
    fn new(url: &str, auth: Auth) -> bitcoincore_rpc::Result<Self>;
}

/// Compatibility implementation for `bitcoincore_rpc::Client`.
impl RpcApiWrapper for bitcoincore_rpc::Client {
    fn new(url: &str, auth: Auth) -> bitcoincore_rpc::Result<Self> {
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
    /// Parameters are just here to match `bitcoincore_rpc::Client::new()`. They
    /// are not used and can be dummy values.
    fn new(_url: &str, _auth: bitcoincore_rpc::Auth) -> bitcoincore_rpc::Result<Self> {
        Ok(Self {
            ledger: Ledger::new(),
        })
    }
}

/// Dumps complete ledger to a string and returns it. This can help identify
/// bugs as it draws a picture of the mock blockchain.
pub fn dump_ledger(rpc: Client) -> String {
    let mut dump = String::new();

    const DELIMETER: &str = "\n-----\n";

    let utxos = rpc.ledger.get_user_utxos();
    let transactions = rpc.ledger.get_transactions();
    let credentials = rpc.ledger.get_credentials();

    dump += format!("UTXO's: {:?}", utxos).as_str();
    dump += DELIMETER;
    dump += format!("Transaction's: {:?}", transactions).as_str();
    dump += DELIMETER;
    dump += format!(
        "Txid's: {:?}",
        transactions
            .iter()
            .map(|tx| tx.compute_txid())
            .collect::<Vec<Txid>>()
    )
    .as_str();
    dump += DELIMETER;
    dump += format!("Credentials's: {:?}", credentials).as_str();

    dump
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
