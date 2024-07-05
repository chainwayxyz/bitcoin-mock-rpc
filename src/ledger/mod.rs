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
    database: Arc<Mutex<Connection>>,
    // /// User's keys and address.
    // credentials: Box<Arc<Mutex<Cell<Vec<UserCredential>>>>>,
    // /// Happened transactions.
    // transactions: Box<Arc<Mutex<Cell<Vec<Transaction>>>>>,
    // /// Unspent transaction outputs, for every addresses.
    // utxos: Box<Arc<Mutex<Cell<HashMap<Address, Vec<OutPoint>>>>>>,
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
                "DROP TABLE IF EXISTS outputs;
                CREATE TABLE outputs
                (
                    tx_id          TEXT              not null,
                    output_id      integer           not null,
                    value          integer           not null,
                    script_pub_key BLOB              not null,
                    is_spent       INTEGER default 0 not null,
                    constraint outputs_pk
                        primary key (tx_id, output_id)
                );
                DROP TABLE IF EXISTS \"transactions\";
                CREATE TABLE \"transactions\"
                (
                    tx_id       TEXT    not null
                        constraint txid
                            primary key,
                    num_outputs integer not null,
                    body        blob    not null
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
