//! # Errors
//!
//! Errors that can be returned from ledger operations.

use thiserror::Error;

/// Ledger error types.
#[derive(Error, Debug)]
pub enum LedgerError {
    #[error("General error: {0}")]
    General(String),
    #[error("Transaction error: {0}")]
    Transaction(String),
    #[error("UTXO error: {0}")]
    Utxo(String),
}

impl From<LedgerError> for bitcoincore_rpc::Error {
    fn from(error: LedgerError) -> Self {
        bitcoincore_rpc::Error::ReturnedError(error.to_string())
    }
}
