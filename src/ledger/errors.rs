//! # Errors
//!
//! Errors that can be returned from ledger operations.

use thiserror::Error;

/// Ledger error types.
#[derive(Error, Debug)]
pub enum LedgerError {
    #[error("Database returned an error: {0}")]
    Database(anyhow::Error),
}

impl From<LedgerError> for bitcoincore_rpc::Error {
    fn from(error: LedgerError) -> Self {
        bitcoincore_rpc::Error::ReturnedError(error.to_string())
    }
}

impl From<anyhow::Error> for LedgerError {
    fn from(error: anyhow::Error) -> Self {
        LedgerError::Database(error)
    }
}
