//! # Block Related Ledger Operations

use super::errors::LedgerError;
use super::Ledger;
use bitcoin::{Transaction, Txid};
use rusqlite::params;
use std::str::FromStr;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

impl Ledger {
    /// Returns current block height.
    ///
    /// # Panics
    ///
    /// Will panic if cannot get height from database.
    pub fn get_block_height(&self) -> u64 {
        self.database
            .lock()
            .unwrap()
            .query_row("SELECT height FROM blocks", params![], |row| {
                let body = row.get::<_, i64>(0).unwrap();

                Ok(body as u64)
            })
            .unwrap()
    }

    /// Returns specified transaction's block height.
    ///
    /// # Panics
    ///
    /// Will panic if cannot get height from database.
    pub fn get_tx_block_height(&self, txid: Txid) -> u64 {
        self.database
            .lock()
            .unwrap()
            .query_row(
                "SELECT (block_height) FROM transactions WHERE txid = ?1",
                params![txid.to_string()],
                |row| {
                    let body = row.get::<_, i64>(0).unwrap();

                    Ok(body as u64)
                },
            )
            .unwrap()
    }

    /// Sets block height to given value.
    ///
    /// # Panics
    ///
    /// Will panic if cannot set height to database.
    fn set_block_height(&self, height: u64) {
        self.database
            .lock()
            .unwrap()
            .execute("UPDATE blocks SET height = ?1", params![height])
            .unwrap();
    }

    /// Increments block height by 1 and sets block time of the next block 10
    /// minutes after the previous block time.
    ///
    /// # Panics
    ///
    /// Will panic if either [`get_block_height`] or [`set_block_height`]
    /// panics.
    pub fn increment_block_height(&self) {
        let last_block_height = self.get_block_height();
        let current_block_height = last_block_height + 1;

        let last_block_time = if last_block_height == 0 {
            // This is genesis block. Use current time.
            let duration = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();

            // Return 10 minutes before current time. New block will have the
            // time of 10 minute after the last block.
            (duration - Duration::from_secs(60 * 10)).as_secs()
        } else {
            self.get_block_time(last_block_height).unwrap()
        };
        let current_block_time = last_block_time + (60 * 10);

        self.set_block_time(current_block_height, current_block_time);
        self.set_block_height(current_block_height);
    }

    /// Gets all the transactions that are in the mempool.
    ///
    /// # Panics
    ///
    /// Will panic if there is a problem with database.
    pub fn get_mempool_transactions(&self) -> Vec<Transaction> {
        // If `txids` is not calculated in a separate scope, there will be a
        // deadlock. Because `get_transaction()` will also try to lock the
        // mutex. So, we do this operation first and unlock mutex for the next
        // call.
        let txids: Vec<Txid> = {
            let db = self.database.lock().unwrap();
            let mut stmt = db.prepare("SELECT (txid) FROM mempool").unwrap();
            let tx_iter = stmt
                .query_map([], |row| {
                    let body: String = row.get(0).unwrap();
                    Ok(Txid::from_str(&body).unwrap())
                })
                .unwrap();
            tx_iter.map(|txid| txid.unwrap()).collect()
        };

        txids
            .iter()
            .map(|txid| self.get_transaction(*txid).unwrap())
            .collect::<Vec<Transaction>>()
    }

    /// Gets a mempool transaction, if it's in the mempool.
    ///
    /// # Panics
    ///
    /// Will panic if there is a problem with database.
    pub fn get_mempool_transaction(&self, txid: Txid) -> Option<Transaction> {
        let mempool_txs = self.get_mempool_transactions();

        mempool_txs
            .iter()
            .find(|tx| {
                if tx.compute_txid() == txid {
                    true
                } else {
                    false
                }
            })
            .cloned()
    }

    /// Cleans up mempool. This should only be called when transactions are
    /// mined.
    ///
    /// # Panics
    ///
    /// Will panic if there is a problem with database.
    pub fn clean_mempool(&self) {
        self.database
            .lock()
            .unwrap()
            .execute("DELETE FROM mempool", params![])
            .unwrap();
    }

    /// Gets `block_height`'th block time, in UNIX format.
    ///
    /// # Panics
    ///
    /// Will panic if there is a problem with database.
    pub fn get_block_time(&self, block_height: u64) -> Result<u64, LedgerError> {
        if let Ok(time) = self.database.lock().unwrap().query_row(
            "SELECT unix_time FROM block_times WHERE block_height = ?1",
            params![block_height],
            |row| {
                let body = row.get::<_, i64>(0).unwrap();

                Ok(body as u64)
            },
        ) {
            return Ok(time);
        };

        Err(LedgerError::BlockInMempool(block_height))
    }

    /// Sets specified blocks time.
    ///
    /// # Panics
    ///
    /// Will panic if there is a problem with database.
    fn set_block_time(&self, block_height: u64, time: u64) {
        self.database
            .lock()
            .unwrap()
            .execute(
                "INSERT INTO block_times (block_height, unix_time) VALUES (?1, ?2)",
                params![block_height, time],
            )
            .unwrap();
    }
}

#[cfg(test)]
mod tests {
    use crate::ledger::Ledger;

    #[test]
    fn get_set_block_height() {
        let ledger = Ledger::new("get_set_block_height");

        let current_height = ledger.get_block_height();
        assert_eq!(current_height, 0);

        ledger.set_block_height(0x45);
        ledger.set_block_time(0x45, 0);
        let current_height = ledger.get_block_height();
        assert_eq!(current_height, 0x45);

        ledger.set_block_height(0x1F);
        ledger.set_block_time(0x1F, 0);
        let current_height = ledger.get_block_height();
        assert_eq!(current_height, 0x1F);
    }

    #[test]
    fn increment_block_height() {
        let ledger = Ledger::new("increment_block_height");

        let current_height = ledger.get_block_height();
        assert_eq!(current_height, 0);

        ledger.increment_block_height();
        let current_height = ledger.get_block_height();
        assert_eq!(current_height, 1);

        // Because we aren't mining blocks rn, we must add block times.
        ledger.set_block_time(0x44, 0);
        ledger.set_block_height(0x45);
        let current_height = ledger.get_block_height();
        assert_eq!(current_height, 0x45);

        // Because we aren't mining blocks rn, we must add block times.
        ledger.set_block_time(0x45, 0);
        ledger.increment_block_height();
        let current_height = ledger.get_block_height();
        assert_eq!(current_height, 0x46);
    }
}
