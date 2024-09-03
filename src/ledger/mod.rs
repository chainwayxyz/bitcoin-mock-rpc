//! # Bitcoin Ledger
//!
//! Mock Bitcoin ledger.
//!
//! This crate is designed to be used as immutable, because of the `RpcApi`'s
//! immutable nature.

use crate::utils;
use rusqlite::Connection;
use std::{
    env,
    sync::{Arc, Mutex},
};

pub mod address;
mod block;
pub(crate) mod errors;
mod script;
mod spending_requirements;
mod transactions;
mod utxo;

/// Mock Bitcoin ledger.
#[derive(Clone, Debug)]
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
    #[tracing::instrument]
    pub fn new(path: &str) -> Self {
        let path = Ledger::get_database_path(path);
        let _ = utils::initialize_logger();

        tracing::trace!("Creating database at path {path}");

        let database = Connection::open(path.clone()).unwrap();

        Ledger::drop_tables(&database).unwrap();
        Ledger::create_tables(&database).unwrap();

        Self {
            database: Arc::new(Mutex::new(database)),
        }
    }

    /// Connects the ledger, previously created by the `new` call. This function
    /// won't clean any data from database. Therefore it is a useful function
    /// for cloning.
    ///
    /// This function is needed because `bitcoincore_rpc` doesn't provide a
    /// `clone` interface. Therefore users of that library need to call `new`
    /// and establish a new connection to the Bitcoin. This is a solution to
    /// that problem: We won't clean any mock data and use previously created
    /// database.
    ///
    /// # Panics
    ///
    /// Panics if SQLite connection can't be established.
    pub fn new_without_cleanup(path: &str) -> Self {
        let path = Ledger::get_database_path(path);

        let database = Connection::open(path.clone()).unwrap();

        Self {
            database: Arc::new(Mutex::new(database)),
        }
    }

    fn get_database_path(path: &str) -> String {
        env::temp_dir().to_str().unwrap().to_owned() + "/bitcoin_mock_rpc_" + path
    }

    fn drop_tables(database: &Connection) -> Result<(), rusqlite::Error> {
        database.execute_batch(
            "
            DROP TABLE IF EXISTS blocks;
            DROP TABLE IF EXISTS mempool;
            DROP TABLE IF EXISTS transactions;
            DROP TABLE IF EXISTS utxos;
            ",
        )
    }

    /// This is where all the ledger data is kept. Note that it is not aimed to
    /// hold all kind of information about the blockchain. Just holds enough
    /// data to provide a persistent storage for the limited features that the
    /// library provides.
    fn create_tables(database: &Connection) -> Result<(), rusqlite::Error> {
        database.execute_batch(
            "
            CREATE TABLE blocks
            (
                height    INTEGER  NOT NULL,
                time      INTEGER  NOT NULL,
                hash      BLOB     NOT NULL,
                coinbase  TEXT     NOT NULL,
                body      BLOB     NOT NULL

                CONSTRAINT height PRIMARY KEY
            );
            INSERT INTO blocks (height, time, hash, coinbase, body) VALUES (0, 500000000, 0, 0, 0);

            CREATE TABLE mempool
            (
                txid  TEXT  NOT NULL

                CONSTRAINT txid PRIMARY KEY
            );

            CREATE TABLE transactions
            (
                txid          TEXT     NOT NULL,
                block_height  INTEGER  NOT NULL,
                body          BLOB     NOT NULL

                CONSTRAINT txid PRIMARY KEY
            );

            CREATE TABLE utxos
            (
                txid          TEXT     NOT NULL,
                vout          INTEGER  NOT NULL
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
