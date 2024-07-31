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
mod script;
mod spending_requirements;
mod transactions;

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
        let path = Ledger::get_database_path(path);

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
        env::temp_dir().to_str().unwrap().to_owned() + "/" + path
    }

    fn drop_tables(database: &Connection) -> Result<(), rusqlite::Error> {
        database.execute_batch(
            "
            DROP TABLE IF EXISTS block_height;
            DROP TABLE IF EXISTS blocks;
            DROP TABLE IF EXISTS tmpblocks;
            DROP TABLE IF EXISTS mempool;
            DROP TABLE IF EXISTS transactions;
            ",
        )
    }

    /// This is where all the ledger data is kept. Note that it is not aimed to
    /// hold all kind of information about the blockchain. Just holds enough
    /// data to provide a persistent storage for the limited features that the
    /// library provides.
    fn create_tables(database: &Connection) -> Result<(), rusqlite::Error> {
        database.execute_batch(
            "CREATE TABLE block_height
            (
                height         INTEGER           not null
            );
            INSERT INTO block_height (height) VALUES (0);

            CREATE TABLE tmpblocks
            (
                height         INTEGER           NOT NULL,
                hash           BLOB              NOT NULL,
                raw_body       BLOB              NOT NULL
                    CONSTRAINT block_height PRIMARY KEY
            );
            INSERT INTO tmpblocks (height, hash, raw_body) VALUES (0, 0, 0);

            CREATE TABLE blocks
            (
                block_height   INTEGER           not null
                    constraint block_height primary key,
                raw_body       BLOB,
                unix_time      INTEGER
            );
            INSERT INTO blocks (block_height, unix_time) VALUES (0, 500000000);

            CREATE TABLE mempool
            (
                txid          TEXT    not null
                    constraint txid primary key
            );

            CREATE TABLE transactions
            (
                txid          TEXT    not null
                    constraint txid primary key,
                block_height  INTEGER not null,
                body          blob    not null
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
