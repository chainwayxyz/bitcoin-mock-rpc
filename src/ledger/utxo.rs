//! # Unspent Trancasction Output Management

use super::Ledger;
use crate::{add_item_to_vec, remove_item_from_vec, return_vec_item};
use bitcoin::OutPoint;

impl Ledger {
    /// Adds a new UTXO to user's UTXO's.
    pub fn _add_utxo(&self, utxo: OutPoint) {
        add_item_to_vec!(self.utxos, utxo);
    }

    /// Removes an UTXO from user's UTXO's.
    pub fn _remove_utxo(&self, utxo: OutPoint) {
        remove_item_from_vec!(self.utxos, utxo);
    }

    /// Returns UTXO's of the user.
    pub fn _get_utxos(&self) -> Vec<OutPoint> {
        return_vec_item!(self.utxos);
    }
}

#[cfg(test)]
mod tests {
    use crate::ledger::Ledger;
    use bitcoin::OutPoint;

    #[test]
    fn add_get_utxos() {
        let ledger = Ledger::new();

        assert_eq!(ledger._get_utxos().len(), 0);

        let dummy_tx = ledger.create_transaction(vec![], vec![]);
        let txid = dummy_tx.compute_txid();

        let utxo = OutPoint { txid, vout: 0 };
        ledger._add_utxo(utxo);

        let utxos = ledger._get_utxos();
        assert_eq!(utxos.len(), 1);
        assert_eq!(*utxos.get(0).unwrap(), utxo);

        let utxo = OutPoint { txid, vout: 1 };
        ledger._add_utxo(utxo);

        let utxos = ledger._get_utxos();
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
        ledger._add_utxo(utxo1);
        let utxo2 = OutPoint { txid, vout: 1 };
        ledger._add_utxo(utxo2);
        let utxo3 = OutPoint { txid, vout: 2 };
        ledger._add_utxo(utxo3);

        let utxos = ledger._get_utxos();
        assert_eq!(*utxos.get(0).unwrap(), utxo1);
        assert_eq!(*utxos.get(1).unwrap(), utxo2);
        assert_eq!(*utxos.get(2).unwrap(), utxo3);
        assert_eq!(utxos.len(), 3);

        let new_utxo = OutPoint { txid, vout: 1 };
        ledger._remove_utxo(new_utxo);

        let utxos = ledger._get_utxos();
        assert_eq!(utxos.len(), 2);
        assert_eq!(*utxos.get(0).unwrap(), utxo1);
        assert_eq!(*utxos.get(1).unwrap(), utxo3);
    }
}
