//! # Transaction Related Ledger Operations

use super::{errors::LedgerError, Ledger};
use crate::{add_item, get_item};
use bitcoin::{Transaction, TxOut, Txid};

impl Ledger {
    /// Adds a new UTXO to user's UTXO's.
    pub fn add_utxo(&self, utxo: TxOut) {
        add_item!(self.utxos, utxo);
    }
    /// Returns UTXO's of the user.
    pub fn _get_utxos(&self) -> Vec<TxOut> {
        get_item!(self.utxos);
    }

    /// Adds transaction to current block, without checking anything.
    pub fn add_transaction_unconditionally(
        &self,
        transaction: Transaction,
    ) -> Result<(), LedgerError> {
        self.database
            .lock()
            .unwrap()
            .insert_transaction_unconditionally(&transaction)?;

        add_item!(self.transactions, transaction);

        Ok(())
    }
    /// Returns user's list of transactions.
    pub fn get_transaction(&self, txid: Txid) -> Result<Transaction, LedgerError> {
        Ok(self
            .database
            .lock()
            .unwrap()
            .get_transaction(&txid.to_string())?)
    }
    /// Returns user's list of transactions.
    pub fn _get_transactions(&self) -> Vec<Transaction> {
        get_item!(self.transactions);
    }
    /// Checks if a transaction is OK or not.
    ///
    /// # Panics
    ///
    /// If mutex can't be locked, it will panic.
    pub fn check_transaction(&self, transaction: Transaction) -> Result<(), LedgerError> {
        Ok(self
            .database
            .lock()
            .unwrap()
            .verify_transaction(&transaction)?)
    }
}

#[cfg(test)]
mod tests {
    use crate::ledger::Ledger;
    use bitcoin::{Amount, TxOut};

    #[test]
    fn add_utxo() {
        let ledger = Ledger::new();

        assert_eq!(ledger._get_utxos().len(), 0);

        // Generate a random address.
        ledger.generate_address();

        // Insert a dummy UTXO.
        let utxo = TxOut {
            value: Amount::from_sat(0x45),
            script_pubkey: ledger._get_address()[0].address.script_pubkey(),
        };
        ledger.add_utxo(utxo);

        assert_eq!(ledger._get_utxos().len(), 1);
        assert_eq!(ledger._get_utxos()[0].value, Amount::from_sat(0x45));
    }
}
