//! # Bitcoin Ledger
//!
//! Mock Bitcoin ledger.
//!
//! This crate is designed to be used as immutable, because of the `RpcApi`'s
//! immutable nature.

use rusqlite::Connection;
use std::{
    env,
    sync::{Arc, Mutex},
};

mod address;
mod block;
mod errors;
mod macros;
mod script;
mod spending_requirements;
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
    /// An SQLite database created at OS's temp directory. Database is named
    /// `path`. This can be used to identify different databases created by
    /// different tests.
    ///
    /// # Panics
    ///
    /// Panics if SQLite connection can't be established and initial query can't
    /// be run.
    pub fn new(path: &str) -> Self {
        let temp_dir = env::temp_dir();
        let path = temp_dir.to_str().unwrap().to_owned() + "/" + path;

        let database = Connection::open(path).unwrap();

        Ledger::drop_databases(&database).unwrap();
        Ledger::create_databases(&database).unwrap();

        Self {
            database: Arc::new(Mutex::new(database)),
        }
    }

    pub fn drop_databases(database: &Connection) -> Result<(), rusqlite::Error> {
        database.execute_batch(
            "
                DROP TABLE IF EXISTS blocks;
                DROP TABLE IF EXISTS transactions;
                DROP TABLE IF EXISTS utxos;
                ",
        )
    }

    pub fn create_databases(database: &Connection) -> Result<(), rusqlite::Error> {
        database.execute_batch(
            "CREATE TABLE blocks
                (
                    height         integer           not null
                );
                INSERT INTO blocks (height) VALUES (0);

                CREATE TABLE transactions
                (
                    txid        TEXT    not null
                        constraint txid primary key,
                    body        blob    not null
                );

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
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new() {
        let _should_not_panic = Ledger::new("ledger_new");
    }
}
