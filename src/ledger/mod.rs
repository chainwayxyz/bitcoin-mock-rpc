//! # Bitcoin Ledger
//!
//! Mock Bitcoin ledger.
//!
//! This crate is designed to be used as immutable, because of the `RpcApi`'s
//! immutable nature.

use rusqlite::Connection;
use std::sync::{Arc, Mutex};

// mod address;
mod errors;
mod macros;
// mod spending_requirements;
mod transactions;
// mod utxo;

/// Mock Bitcoin ledger.
#[derive(Clone)]
pub struct Ledger {
    /// Database connection.
    database: Arc<Mutex<Connection>>,
}

impl Ledger {
    /// Creates a new empty ledger.
    ///
    /// # Panics
    ///
    /// Panics if SQLite connection can't be established and initial query can't
    /// be run.
    pub fn new() -> Self {
        let database = Connection::open_in_memory().unwrap();

        database
            .execute_batch(
                "
                DROP TABLE IF EXISTS \"transactions\";
                CREATE TABLE \"transactions\"
                (
                    txid        TEXT    not null
                        constraint txid primary key,
                    body        blob    not null
                );

                DROP TABLE IF EXISTS utxos;
                CREATE TABLE utxos
                (
                    txid           TEXT              not null,
                    vout           integer           not null,
                    value          integer           not null,
                    script_pubkey  BLOB              not null,
                    is_spent       INTEGER default 0 not null,
                    constraint utxos_pk
                        primary key (txid, vout)
                );
                ",
            )
            .unwrap();

        Self {
            database: Arc::new(Mutex::new(database)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new() {
        let _should_not_panic = Ledger::new();
    }
}
