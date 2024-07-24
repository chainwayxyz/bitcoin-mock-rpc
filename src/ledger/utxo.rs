//! # Unspent Transaction Output Related Operations

use super::Ledger;
use bitcoin::{absolute, OutPoint};
use rusqlite::params;

impl Ledger {
    /// Adds a new UTXO with the time lock. Accepts absolute time lock.
    ///
    /// # Panics
    ///
    /// Will panic if cannot set UTXO to database.
    pub fn add_utxo_with_lock_time(&self, utxo: OutPoint, lock: absolute::LockTime) {
        let sequence = lock.to_consensus_u32();

        self.database
            .lock()
            .unwrap()
            .execute(
                "INSERT INTO \"utxos\" (txid, vout, sequence) VALUES (?1, ?2, ?3)",
                params![utxo.txid.to_string(), utxo.vout, sequence],
            )
            .unwrap();
    }

    /// Returns UTXO's timelock, if present.
    pub fn get_utxo_timelock(&self, utxo: OutPoint) -> Option<absolute::LockTime> {
        if let Ok(sequence) = self.database.lock().unwrap().query_row(
            "SELECT sequence FROM utxos WHERE txid = ?1 AND vout = ?2",
            params![utxo.txid.to_string(), utxo.vout],
            |row| Ok(row.get::<_, u32>(0).unwrap()),
        ) {
            return Some(absolute::LockTime::from_consensus(sequence));
        };

        None
    }

    /// Removes an UTXO.
    pub fn remove_utxo(&self, _utxo: OutPoint) {
        unimplemented!()
    }
}

#[cfg(test)]
mod tests {
    use crate::ledger::Ledger;
    use bitcoin::{
        absolute::{self, Height},
        Amount, OutPoint, ScriptBuf,
    };

    #[test]
    fn add_get_utxo() {
        let ledger = Ledger::new("add_get_utxo");

        let txout = ledger.create_txout(Amount::from_sat(0x45), ScriptBuf::new());
        let tx = ledger.create_transaction(vec![], vec![txout]);
        let txid = tx.compute_txid();
        let vout = 0;
        let utxo0 = OutPoint { txid, vout };
        let time_lock0: absolute::LockTime =
            absolute::LockTime::Blocks(Height::from_consensus(0x45).unwrap());

        assert_eq!(ledger.get_utxo_timelock(utxo0), None);

        ledger.add_utxo_with_lock_time(utxo0, time_lock0);
        assert_eq!(ledger.get_utxo_timelock(utxo0), Some(time_lock0));

        let txout = ledger.create_txout(Amount::from_sat(0x100), ScriptBuf::new());
        let tx = ledger.create_transaction(vec![], vec![txout]);
        let txid = tx.compute_txid();
        let vout = 0;
        let utxo1 = OutPoint { txid, vout };
        let time_lock1: absolute::LockTime =
            absolute::LockTime::Blocks(Height::from_consensus(0x1F).unwrap());

        assert_eq!(ledger.get_utxo_timelock(utxo1), None);
        ledger.add_utxo_with_lock_time(utxo1, time_lock1);
        assert_eq!(ledger.get_utxo_timelock(utxo1), Some(time_lock1));

        assert_eq!(ledger.get_utxo_timelock(utxo0), Some(time_lock0));
    }
}
