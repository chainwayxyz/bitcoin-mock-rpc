//! # Unspent Transaction Output Management

use super::{errors::LedgerError, Ledger};
use crate::{add_item_to_vec, remove_item_from_vec, return_vec_item};
use bitcoin::{Amount, OutPoint};

impl Ledger {
    /// Adds a new UTXO to user's UTXO's.
    pub fn add_utxo(&self, utxo: OutPoint) {
        add_item_to_vec!(self.utxos, utxo);
    }

    /// Removes an UTXO from user's UTXO's.
    pub fn remove_utxo(&self, utxo: OutPoint) {
        remove_item_from_vec!(self.utxos, utxo);
    }

    /// Returns UTXO's of the user.
    pub fn get_utxos(&self) -> Vec<OutPoint> {
        return_vec_item!(self.utxos);
    }

    /// Calculate balance using UTXO's.
    pub fn calculate_balance(&self) -> Result<Amount, LedgerError> {
        let mut amount = Amount::from_sat(0);

        for utxo in self.get_utxos() {
            let tx = self.get_transaction(utxo.txid)?;

            let txout = tx
                .output
                .get(utxo.vout as usize)
                .ok_or(LedgerError::UTXO(format!(
                    "vout {} couldn't be found in transaction with txid {}",
                    utxo.vout, utxo.txid
                )))?;

            amount += txout.value;
        }

        Ok(amount)
    }
}

#[cfg(test)]
mod tests {
    use crate::{add_item_to_vec, ledger::Ledger, remove_item_from_vec};
    use bitcoin::{Amount, OutPoint};

    #[test]
    fn add_get_utxos() {
        let ledger = Ledger::new();

        assert_eq!(ledger.get_utxos().len(), 0);

        let dummy_tx = ledger.create_transaction(vec![], vec![]);
        let txid = dummy_tx.compute_txid();

        let utxo = OutPoint { txid, vout: 0 };
        ledger.add_utxo(utxo);

        let utxos = ledger.get_utxos();
        assert_eq!(utxos.len(), 1);
        assert_eq!(*utxos.get(0).unwrap(), utxo);

        let utxo = OutPoint { txid, vout: 1 };
        ledger.add_utxo(utxo);

        let utxos = ledger.get_utxos();
        assert_eq!(utxos.len(), 2);
        assert_ne!(*utxos.get(0).unwrap(), utxo);
        assert_eq!(*utxos.get(1).unwrap(), utxo);
    }

    #[test]
    fn add_remove_utxos() {
        let ledger = Ledger::new();

        let dummy_tx = ledger.create_transaction(vec![], vec![]);
        let txid = dummy_tx.compute_txid();

        let utxo1 = OutPoint { txid, vout: 0 };
        ledger.add_utxo(utxo1);
        let utxo2 = OutPoint { txid, vout: 1 };
        ledger.add_utxo(utxo2);
        let utxo3 = OutPoint { txid, vout: 2 };
        ledger.add_utxo(utxo3);

        let utxos = ledger.get_utxos();
        assert_eq!(*utxos.get(0).unwrap(), utxo1);
        assert_eq!(*utxos.get(1).unwrap(), utxo2);
        assert_eq!(*utxos.get(2).unwrap(), utxo3);
        assert_eq!(utxos.len(), 3);

        let new_utxo = OutPoint { txid, vout: 1 };
        ledger.remove_utxo(new_utxo);

        let utxos = ledger.get_utxos();
        assert_eq!(utxos.len(), 2);
        assert_eq!(*utxos.get(0).unwrap(), utxo1);
        assert_eq!(*utxos.get(1).unwrap(), utxo3);
    }

    #[test]
    fn calculate_balance() {
        let ledger = Ledger::new();

        assert_eq!(ledger.calculate_balance().unwrap(), Amount::from_sat(0));

        // Insert dummy transactions.
        let txout = ledger.create_txout(100 - 0x1F, None);
        let tx = ledger.create_transaction(vec![], vec![txout]);
        let txid = tx.compute_txid();
        let utxo = OutPoint { txid, vout: 0 };
        add_item_to_vec!(ledger.utxos, utxo);
        add_item_to_vec!(ledger.transactions, tx);

        let txout = ledger.create_txout(0x1F, None);
        let tx = ledger.create_transaction(vec![], vec![txout]);
        let txid = tx.compute_txid();
        let utxo = OutPoint { txid, vout: 0 };
        add_item_to_vec!(ledger.utxos, utxo);
        add_item_to_vec!(ledger.transactions, tx);

        let txout1 = ledger.create_txout(100 - 0x1F, None);
        let txout2 = ledger.create_txout(0x1F, None);
        let tx = ledger.create_transaction(vec![], vec![txout1, txout2]);
        let txid = tx.compute_txid();
        let utxo1 = OutPoint { txid, vout: 0 };
        let utxo2 = OutPoint { txid, vout: 1 };
        add_item_to_vec!(ledger.utxos, utxo1);
        add_item_to_vec!(ledger.utxos, utxo2);
        add_item_to_vec!(ledger.transactions, tx);

        // Balance should be equal to 200 Sats.
        assert_eq!(ledger.calculate_balance().unwrap(), Amount::from_sat(200));

        // Spend one UTXO.
        remove_item_from_vec!(ledger.utxos, utxo2);

        // Balance should be equal to 200 - 0x1F Sats.
        assert_eq!(
            ledger.calculate_balance().unwrap(),
            Amount::from_sat(200 - 0x1F)
        );
    }
}
