//! # Block Related Ledger Operations

use super::Ledger;
use bitcoin::Txid;
use rusqlite::params;

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
