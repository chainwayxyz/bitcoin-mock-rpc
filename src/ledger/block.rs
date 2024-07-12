//! # Block Related Ledger Operations

use super::Ledger;
use bitcoin::{Transaction, Txid};
use rusqlite::params;
use std::str::FromStr;

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
    pub fn set_block_height(&self, height: u64) {
        self.database
            .lock()
            .unwrap()
            .execute("UPDATE blocks SET height = ?1", params![height])
            .unwrap();
    }

    /// Increments block height by 1.
    ///
    /// # Panics
    ///
    /// Will panic if either [`get_block_height`] or [`set_block_height`]
    /// panics.
    pub fn increment_block_height(&self) {
        let current_height = self.get_block_height();
        self.set_block_height(current_height + 1);
    }

    /// Gets all the transactions that are in the mempool.
    ///
    /// # Panics
    ///
    /// Will panic if there is a problem with database.
    pub fn get_mempool_transactions(&self) -> Vec<Transaction> {
        let db = self.database.lock().unwrap();
        let mut stmt = db.prepare("SELECT (txid) FROM mempool").unwrap();
        let tx_iter = stmt
            .query_map([], |row| {
                let body: String = row.get(0).unwrap();
                Ok(Txid::from_str(&body).unwrap())
            })
            .unwrap();
        let txids: Vec<Txid> = tx_iter.map(|txid| txid.unwrap()).collect();

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
        let current_height = ledger.get_block_height();
        assert_eq!(current_height, 0x45);

        ledger.set_block_height(0x1F);
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

        ledger.set_block_height(0x45);
        let current_height = ledger.get_block_height();
        assert_eq!(current_height, 0x45);

        ledger.increment_block_height();
        let current_height = ledger.get_block_height();
        assert_eq!(current_height, 0x46);
    }
}
