//! # Unspent Trancasction Outputs
//!
//! This crate manages UTXO's.

use super::Ledger;
use crate::{add_item_to_vec, remove_item_from_vec, return_vec_item};
use bitcoin::OutPoint;

impl Ledger {
    /// Adds a new UTXO to user's UTXO's.
    pub fn _add_utxo(&self, utxo: OutPoint) {
        add_item_to_vec!(self.utxos, utxo);
    }
    /// Removes an UTXO, when it's spent.
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
    fn add_remove_utxos() {
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
}
