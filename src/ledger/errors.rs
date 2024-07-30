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
    #[error("SpendingRequirements error: {0}")]
    SpendingRequirements(String),
    #[error("Script error: {0}")]
    Script(String),
    #[error("Requested block is in mempool; Block height: {0}")]
    BlockInMempool(u32),
}

impl From<LedgerError> for bitcoincore_rpc::Error {
    fn from(error: LedgerError) -> Self {
        bitcoincore_rpc::Error::ReturnedError(error.to_string())
    }
}
