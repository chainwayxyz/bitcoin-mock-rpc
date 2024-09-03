//! # UTXO Management

use super::{errors::LedgerError, Ledger};
use bitcoin::OutPoint;
use rusqlite::params;

impl Ledger {
    pub fn add_utxo(&self, utxo: OutPoint) -> Result<(), LedgerError> {
        if let Err(e) = self.database.lock().unwrap().execute(
            "INSERT INTO utxos (txid, vout) VALUES (?1, ?2)",
            params![utxo.txid.to_string(), utxo.vout],
        ) {
            return Err(LedgerError::Transaction(format!(
                "Couldn't add utxo {:?} to ledger: {}",
                utxo, e
            )));
        };
        tracing::trace!("UTXO {utxo:?} saved");

        Ok(())
    }

    pub fn is_utxo_spent(&self, utxo: OutPoint) -> bool {
        self.database
            .lock()
            .unwrap()
            .query_row(
                "SELECT * FROM utxos WHERE txid = ?1 AND vout = ?2",
                params![utxo.txid.to_string(), utxo.vout],
                |_| Ok(()),
            )
            .is_err()
    }

    pub fn remove_utxo(&self, utxo: OutPoint) -> Result<(), LedgerError> {
        if let Err(e) = self.database.lock().unwrap().execute(
            "DELETE FROM utxos WHERE txid = ?1 AND vout = ?2",
            params![utxo.txid.to_string(), utxo.vout],
        ) {
            return Err(LedgerError::Transaction(format!(
                "Couldn't remove utxo {:?} from ledger: {}",
                utxo, e
            )));
        };
        tracing::trace!("UTXO {utxo:?} marked as spent");

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::ledger::Ledger;
    use bitcoin::{hashes::Hash, OutPoint, Txid};

    #[test]
    fn basic_add_remove_utxo() {
        let ledger = Ledger::new("basic_add_remove_utxo");

        let utxo = OutPoint {
            txid: Txid::all_zeros(),
            vout: 0x45,
        };

        assert!(ledger.is_utxo_spent(utxo));

        ledger.add_utxo(utxo).unwrap();
        assert!(!ledger.is_utxo_spent(utxo));

        ledger.remove_utxo(utxo).unwrap();
        assert!(ledger.is_utxo_spent(utxo));
    }
}
