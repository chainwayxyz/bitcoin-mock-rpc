//! # Bitcoin Ledger
//!
//! Mock Bitcoin ledger.
//!
//! This crate is designed to be used as immutable, because of the `RpcApi`'s
//! immutable nature.

use crate::utils;
use rusqlite::{params, Connection};
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

        let database = Connection::open(path.clone()).unwrap();

        // If database has another connections, skip clearing.
        if Ledger::get_database_connection_count(&database) == 0 {
            tracing::trace!("Creating new database at path {path}");

            Ledger::drop_tables(&database).unwrap();
            Ledger::create_tables(&database).unwrap();
        }
        Ledger::increment_connection_count(&database);

        tracing::trace!("Database connection to {path} is established");

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

        tracing::trace!("Connecting to the existing database {path} without resetting");

        Self {
            database: Arc::new(Mutex::new(database)),
        }
    }

    /// Returns current connection count to the database. If not zero
    fn get_database_connection_count(database: &Connection) -> i64 {
        let count = database.query_row("SELECT count FROM connection_info", params![], |row| {
            Ok(row.get::<_, i64>(0).unwrap())
        });

        let count = count.unwrap_or(0);
        tracing::trace!("Current connection count: {count}");

        count
    }

    /// Increments connection count.
    fn increment_connection_count(database: &Connection) {
        let count = Self::get_database_connection_count(database) + 1;
        tracing::trace!("Incrementing connection count to {count}...");

        database
            .execute("UPDATE connection_info SET count = ?1", params![count])
            .unwrap();
    }

    /// Decrements connection count.
    fn decrement_connection_count(&self) {
        let count = Self::get_database_connection_count(&self.database.lock().unwrap()) - 1;
        tracing::trace!("Decrementing connection count to {count}...");

        self.database
            .lock()
            .unwrap()
            .execute("UPDATE connection_info SET count = ?1", params![count])
            .unwrap();
    }

    fn get_database_path(path: &str) -> String {
        env::temp_dir().to_str().unwrap().to_owned() + "/bitcoin_mock_rpc_" + path
    }

    fn drop_tables(database: &Connection) -> Result<(), rusqlite::Error> {
        database.execute_batch(
            "
            DROP TABLE IF EXISTS connection_info;
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
            CREATE TABLE connection_info
            (
                count  INTEGER  NOT NULL

                CONSTRAINT count PRIMARY KEY
            );
            INSERT INTO connection_info (count) VALUES (0);

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

impl Drop for Ledger {
    fn drop(&mut self) {
        self.decrement_connection_count();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new() {
        let _should_not_panic = Ledger::new("ledger_new");
    }

    #[test]
    fn concurrent_connections() {
        let ledger = Ledger::new("concurrent_connections");

        let ledger2 = Ledger::new("concurrent_connections");

        let count = Ledger::get_database_connection_count(&ledger.database.lock().unwrap());
        let count2 = Ledger::get_database_connection_count(&ledger2.database.lock().unwrap());

        assert_eq!(count, count2);
        assert_eq!(count, 2);
    }

    #[test]
    fn concurrent_connection_panics() {
        let ledger = Ledger::new("concurrent_connection_panics");

        std::panic::set_hook(Box::new(|_info| {
            // do nothing
        }));

        std::thread::spawn(|| {
            let _ledger2 = Ledger::new("concurrent_connection_panics");

            let _result = std::panic::catch_unwind(|| {
                panic!("test panic");
            });
        })
        .join()
        .unwrap();

        let _ledger3 = Ledger::new("concurrent_connection_panics");

        let count = Ledger::get_database_connection_count(&ledger.database.lock().unwrap());
        assert_eq!(count, 2);
    }
}
