//! # Errors
//!
//! Errors that can be returned from ledger operations.

use thiserror::Error;

/// Ledger error types.
#[derive(Error, Debug)]
pub enum LedgerError {
    #[error("Transaction error: {0}")]
    Transaction(String),
    #[error("UTXO error: {0}")]
    Utxo(String),
    #[error("Anyhow error: {0}")]
    AnyHow(anyhow::Error),
}

impl From<LedgerError> for bitcoincore_rpc::Error {
    fn from(error: LedgerError) -> Self {
        bitcoincore_rpc::Error::ReturnedError(error.to_string())
    }
}

impl From<anyhow::Error> for LedgerError {
    fn from(error: anyhow::Error) -> Self {
        LedgerError::AnyHow(error)
    }
}
