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
