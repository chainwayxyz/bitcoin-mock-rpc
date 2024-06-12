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

    /// Adds transaction to current block, after verifying.
    pub fn add_transaction(&self, transaction: Transaction) -> Result<Txid, LedgerError> {
        self.check_transaction(&transaction)?;

        self.add_transaction_unconditionally(transaction)
    }
    /// Adds transaction to current block, without checking anything.
    pub fn add_transaction_unconditionally(
        &self,
        transaction: Transaction,
    ) -> Result<Txid, LedgerError> {
        self.database
            .lock()
            .unwrap()
            .insert_transaction_unconditionally(&transaction)?;

        add_item!(self.transactions, transaction.clone());

        Ok(transaction.compute_txid())
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
    /// Checks if a transaction is valid or not.
    ///
    /// # Panics
    ///
    /// If mutex can't be locked, it will panic.
    pub fn check_transaction(&self, transaction: &Transaction) -> Result<(), LedgerError> {
        Ok(self
            .database
            .lock()
            .unwrap()
            .verify_transaction(transaction)?)
    }
}

#[cfg(test)]
mod tests {
    use crate::{ledger::Ledger, test_common};
    use bitcoin::{Amount, ScriptBuf, TxOut};

    /// Tests UTXO operations over ledger.
    #[test]
    fn utxo() {
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

    /// Tests transaction operations over ledger, without any rule checks.
    #[test]
    fn transactions_without_checking() {
        let ledger = Ledger::new();

        assert_eq!(ledger._get_transactions().len(), 0);

        let txout = TxOut {
            value: Amount::from_sat(0x45),
            script_pubkey: ScriptBuf::new(),
        };
        let tx = test_common::create_transaction(vec![], vec![txout]);
        let txid = tx.compute_txid();

        assert_eq!(
            txid,
            ledger.add_transaction_unconditionally(tx.clone()).unwrap()
        );

        let txs = ledger._get_transactions();
        assert_eq!(txs.len(), 1);

        let tx2 = txs.get(0).unwrap().to_owned();
        assert_eq!(tx, tx2);

        let tx2 = ledger.get_transaction(txid).unwrap();
        assert_eq!(tx, tx2);
    }

    /// Tests transaction operations over ledger, with rule checks.
    #[test]
    fn transactions_with_checks() {
        let ledger = Ledger::new();

        assert_eq!(ledger._get_transactions().len(), 0);

        let txout = TxOut {
            value: Amount::from_sat(0x45 * 0x45),
            script_pubkey: ScriptBuf::new(),
        };
        let tx = test_common::create_transaction(vec![], vec![txout.clone()]);
        let txid = tx.compute_txid();

        // First add some funds to user.
        assert_eq!(
            txid,
            ledger.add_transaction_unconditionally(tx.clone()).unwrap()
        );

        // Input amount is zero. This should not be accepted.
        if let Ok(_) = ledger.add_transaction(tx.clone()) {
            assert!(false);
        };

        let txin = test_common::create_txin(txid);
        let tx = test_common::create_transaction(vec![txin], vec![txout]);
        let txid = tx.compute_txid();

        // Input amount is OK. This should be accepted.
        assert_eq!(txid, ledger.add_transaction(tx.clone()).unwrap());

        let txs = ledger._get_transactions();
        assert_eq!(txs.len(), 2);

        let tx2 = txs.get(1).unwrap().to_owned();
        assert_eq!(tx, tx2);

        let tx2 = ledger.get_transaction(txid).unwrap();
        assert_eq!(tx, tx2);
    }
}
