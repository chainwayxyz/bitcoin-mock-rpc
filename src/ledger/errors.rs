//! # Errors
//!
//! Errors that can be returned from ledger operations.

use thiserror::Error;

/// Ledger error types.
#[derive(Error, Debug)]
pub enum LedgerError {
    #[error("Ledger returned a general error: {0}")]
    General(String),
    #[error("Transaction is not OK: {0}")]
    Transaction(String),
    #[error("UTXO cannot be spend: {0}")]
    UTXO(String),
}

impl From<LedgerError> for bitcoincore_rpc::Error {
    fn from(error: LedgerError) -> Self {
        bitcoincore_rpc::Error::ReturnedError(error.to_string())
    }
}
