//! # Unspent Transaction Output Management

use super::{errors::LedgerError, Ledger};
use crate::{add_utxo_to_address, get_utxos_for_address, remove_utxo_from_address};
use bitcoin::{Address, Amount, OutPoint};

impl Ledger {
    /// Adds a new UTXO to `address`'es UTXO's.
    pub fn add_utxo(&self, address: Address, utxo: OutPoint) {
        add_utxo_to_address!(self.utxos, address, utxo);
    }

    /// Removes an UTXO from user's UTXO's.
    pub fn remove_utxo(&self, address: Address, utxo: OutPoint) {
        remove_utxo_from_address!(self.utxos, address, utxo);
    }

    /// Returns UTXO's of an address.
    pub fn get_utxos(&self, address: Address) -> Vec<OutPoint> {
        get_utxos_for_address!(self.utxos, &address, utxos);

        match utxos {
            Some(utxos) => utxos.to_owned(),
            None => Vec::new(),
        }
    }

    /// Returns UTXO's of the user.
    pub fn get_user_utxos(&self) -> Vec<OutPoint> {
        let mut ret = Vec::new();

        self.get_credentials().iter().for_each(|credential| {
            self.get_utxos(credential.address.clone())
                .iter()
                .for_each(|utxo| {
                    ret.push(utxo.to_owned());
                });
        });

        ret
    }

    /// Combines UTXO's which equals or more of the specified amount. This is
    /// useful for generating TxIn's.
    ///
    /// UTXO's are selected with FIFO technique. TODO: This can be optimized later.
    ///
    /// # Returns
    ///
    /// Returns UTXO's in a `Vec` and their total value.
    pub fn _combine_utxos(&self, amount: Amount) -> Result<(Vec<OutPoint>, Amount), LedgerError> {
        let mut total_value = Amount::from_sat(0);
        let mut utxos = Vec::new();

        for utxo in self.get_user_utxos() {
            let tx = self.get_transaction(utxo.txid)?;
            let txout = tx.output.get(utxo.vout as usize).unwrap();

            total_value += txout.value;
            utxos.push(utxo);

            if total_value >= amount {
                break;
            }
        }

        if amount > total_value {
            return Err(LedgerError::Utxo(format!(
                "Requested amount bigger than balance: {amount} > {total_value}"
            )));
        }

        Ok((utxos, total_value))
    }

    /// Calculate balance using UTXO's.
    pub fn calculate_balance(&self) -> Result<Amount, LedgerError> {
        let mut amount = Amount::from_sat(0);

        for utxo in self.get_user_utxos() {
            let tx = self.get_transaction(utxo.txid)?;

            let txout = tx
                .output
                .get(utxo.vout as usize)
                .ok_or(LedgerError::Utxo(format!(
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
    use crate::{add_item_to_vec, add_utxo_to_address, ledger::Ledger, remove_utxo_from_address};
    use bitcoin::{Amount, OutPoint};

    #[test]
    fn add_get_utxos() {
        let ledger = Ledger::new();
        let address = Ledger::generate_credential().address;

        assert_eq!(ledger.get_utxos(address.clone()).len(), 0);

        let txout = ledger.create_txout(Amount::from_sat(0x45), Some(address.script_pubkey()));
        let tx = ledger.create_transaction(vec![], vec![txout]);
        let txid = tx.compute_txid();

        let utxo = OutPoint { txid, vout: 0 };
        ledger.add_utxo(address.clone(), utxo);

        let utxos = ledger.get_utxos(address.clone());
        assert_eq!(utxos.len(), 1);
        assert_eq!(*utxos.get(0).unwrap(), utxo);

        let utxo = OutPoint { txid, vout: 1 };
        ledger.add_utxo(address.clone(), utxo);

        let utxos = ledger.get_utxos(address.clone());
        assert_eq!(utxos.len(), 2);
        assert_ne!(*utxos.get(0).unwrap(), utxo);
        assert_eq!(*utxos.get(1).unwrap(), utxo);
    }

    #[test]
    fn add_get_user_utxos() {
        let ledger = Ledger::new();

        let credential0 = Ledger::generate_credential();
        ledger.add_credential(credential0.clone());
        let address0 = credential0.address;

        let credential1 = Ledger::generate_credential();
        ledger.add_credential(credential1.clone());
        let address1 = credential1.address;

        assert_eq!(ledger.get_user_utxos().len(), 0);

        let txout = ledger.create_txout(Amount::from_sat(0x45), Some(address0.script_pubkey()));
        let tx = ledger.create_transaction(vec![], vec![txout]);
        let txid = tx.compute_txid();
        let utxo = OutPoint { txid, vout: 0 };
        ledger.add_utxo(address0.clone(), utxo);

        let utxos = ledger.get_user_utxos();
        assert_eq!(utxos.len(), 1);
        assert_eq!(*utxos.get(0).unwrap(), utxo);

        let utxo = OutPoint { txid, vout: 1 };
        ledger.add_utxo(address0.clone(), utxo);

        let utxos = ledger.get_user_utxos();
        assert_eq!(utxos.len(), 2);
        assert_ne!(*utxos.get(0).unwrap(), utxo);
        assert_eq!(*utxos.get(1).unwrap(), utxo);

        let utxo = OutPoint { txid, vout: 2 };
        ledger.add_utxo(address1.clone(), utxo);

        let utxos = ledger.get_user_utxos();
        assert_eq!(utxos.len(), 3);
        assert_ne!(*utxos.get(0).unwrap(), utxo);
        assert_ne!(*utxos.get(1).unwrap(), utxo);
        assert_eq!(*utxos.get(2).unwrap(), utxo);
    }

    #[test]
    fn add_remove_utxos() {
        let ledger = Ledger::new();
        let address = Ledger::generate_credential().address;

        let tx = ledger.create_transaction(vec![], vec![]);
        let txid = tx.compute_txid();

        let utxo1 = OutPoint { txid, vout: 0 };
        ledger.add_utxo(address.clone(), utxo1);
        let utxo2 = OutPoint { txid, vout: 1 };
        ledger.add_utxo(address.clone(), utxo2);
        let utxo3 = OutPoint { txid, vout: 2 };
        ledger.add_utxo(address.clone(), utxo3);

        let utxos = ledger.get_utxos(address.clone());
        assert_eq!(*utxos.get(0).unwrap(), utxo1);
        assert_eq!(*utxos.get(1).unwrap(), utxo2);
        assert_eq!(*utxos.get(2).unwrap(), utxo3);
        assert_eq!(utxos.len(), 3);

        let new_utxo = OutPoint { txid, vout: 1 };
        ledger.remove_utxo(address.clone(), new_utxo);

        let utxos = ledger.get_utxos(address.clone());
        assert_eq!(utxos.len(), 2);
        assert_eq!(*utxos.get(0).unwrap(), utxo1);
        assert_eq!(*utxos.get(1).unwrap(), utxo3);
    }

    #[test]
    fn calculate_balance() {
        let ledger = Ledger::new();
        let credential = Ledger::generate_credential();
        ledger.add_credential(credential.clone());
        let address = credential.address;

        assert_eq!(ledger.calculate_balance().unwrap(), Amount::from_sat(0));

        let txout = ledger.create_txout(Amount::from_sat(100 - 0x1F), None);
        let tx = ledger.create_transaction(vec![], vec![txout]);
        let txid = tx.compute_txid();
        let utxo = OutPoint { txid, vout: 0 };
        add_utxo_to_address!(ledger.utxos, address.clone(), utxo);
        add_item_to_vec!(ledger.transactions, tx);

        let txout = ledger.create_txout(Amount::from_sat(0x1F), None);
        let tx = ledger.create_transaction(vec![], vec![txout]);
        let txid = tx.compute_txid();
        let utxo = OutPoint { txid, vout: 0 };
        add_utxo_to_address!(ledger.utxos, address.clone(), utxo);
        add_item_to_vec!(ledger.transactions, tx);

        let txout1 = ledger.create_txout(Amount::from_sat(100 - 0x1F), None);
        let txout2 = ledger.create_txout(Amount::from_sat(0x1F), None);
        let tx = ledger.create_transaction(vec![], vec![txout1, txout2]);
        let txid = tx.compute_txid();
        let utxo1 = OutPoint { txid, vout: 0 };
        let utxo2 = OutPoint { txid, vout: 1 };
        add_utxo_to_address!(ledger.utxos, address.clone(), utxo1);
        add_utxo_to_address!(ledger.utxos, address.clone(), utxo2);
        add_item_to_vec!(ledger.transactions, tx);

        // Balance should be equal to 200 Sats.
        assert_eq!(ledger.calculate_balance().unwrap(), Amount::from_sat(200));

        // Spend one UTXO.
        remove_utxo_from_address!(ledger.utxos, address.clone(), utxo2);

        // Balance should be equal to 200 - 0x1F Sats.
        assert_eq!(
            ledger.calculate_balance().unwrap(),
            Amount::from_sat(200 - 0x1F)
        );
    }

    #[test]
    #[ignore]
    fn combine_utxos() {
        let ledger = Ledger::new();

        let credential = Ledger::generate_credential_from_witness();
        ledger.add_credential(credential.clone());
        let address = credential.address;

        // Add some small UTXO's to user.
        for i in 0..100 {
            let txout = ledger.create_txout(Amount::from_sat(i), Some(address.script_pubkey()));
            let tx = ledger.create_transaction(vec![], vec![txout]);

            ledger.add_transaction_unconditionally(tx).unwrap();
        }

        // Because combining currently uses FIFO algorithm for choosing UTXO's
        // and we know what are getting pushed, we can guess correct txin value.
        assert_eq!(
            ledger._combine_utxos(Amount::from_sat(1)).unwrap().1,
            Amount::from_sat(1)
        );
        assert_eq!(
            ledger._combine_utxos(Amount::from_sat(4)).unwrap().1,
            Amount::from_sat(6)
        );
        assert_eq!(
            ledger._combine_utxos(Amount::from_sat(10)).unwrap().1,
            Amount::from_sat(10)
        );
        assert_eq!(
            ledger._combine_utxos(Amount::from_sat(11)).unwrap().1,
            Amount::from_sat(15)
        );

        // Trying to request an amount bigger than current balance should throw
        // an error.
        if let Ok(_) = ledger._combine_utxos(Amount::from_sat((0..100).sum::<u64>() + 1)) {
            assert!(false);
        }
    }
}
