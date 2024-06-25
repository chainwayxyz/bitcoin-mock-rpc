//! # Transaction Related Ledger Operations

use super::{errors::LedgerError, Ledger};
use crate::{add_item_to_vec, get_item, ledger::address::UserCredential, return_vec_item};
use bitcoin::{absolute, Amount, OutPoint, ScriptBuf, Transaction, TxIn, TxOut, Txid, Witness};

impl Ledger {
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
        // Add transaction to list.
        add_item_to_vec!(self.transactions, transaction.clone());

        // TODO: Add UTXO's to list. Careful about only adding if address is
        // user's.
        transaction.output.iter().for_each(|_utxo| {});

        Ok(transaction.compute_txid())
    }
    /// Returns a transaction which matches the given txid.
    pub fn get_transaction(&self, txid: Txid) -> Result<Transaction, LedgerError> {
        let txs: Vec<Transaction>;
        get_item!(self.transactions, txs);

        let tx = txs
            .iter()
            .find(|tx| tx.compute_txid() == txid)
            .ok_or(LedgerError::Transaction(String::from(
                "No transaction is matched with ".to_string() + txid.to_string().as_str(),
            )))?
            .to_owned();

        Ok(tx)
    }
    /// Returns user's list of transactions.
    pub fn get_transactions(&self) -> Vec<Transaction> {
        return_vec_item!(self.transactions);
    }

    /// Checks if a transaction is valid or not.
    pub fn check_transaction(&self, transaction: &Transaction) -> Result<(), LedgerError> {
        let balance = self.calculate_balance()?;
        let out_value = self.calculate_transaction_output_value(transaction.clone());

        if balance < out_value {
            return Err(LedgerError::Transaction(format!(
                "Balance: {} is not enough for: {}",
                balance, out_value
            )));
        }

        Ok(())
    }

    pub fn create_txin(&self, txid: Txid) -> TxIn {
        let credentials: Vec<UserCredential>;
        get_item!(self.credentials, credentials);
        let witness = match credentials.last() {
            Some(c) => match c.to_owned().witness {
                Some(w) => w,
                None => Witness::new(),
            },
            None => Witness::new(),
        };

        TxIn {
            previous_output: OutPoint { txid, vout: 0 },
            witness,
            ..Default::default()
        }
    }

    pub fn create_txout(&self, satoshi: u64, script_pubkey: Option<ScriptBuf>) -> TxOut {
        TxOut {
            value: Amount::from_sat(satoshi),
            script_pubkey: match script_pubkey {
                Some(script_pubkey) => script_pubkey,
                None => ScriptBuf::new(),
            },
        }
    }

    pub fn create_transaction(&self, tx_ins: Vec<TxIn>, tx_outs: Vec<TxOut>) -> Transaction {
        bitcoin::Transaction {
            version: bitcoin::transaction::Version(2),
            lock_time: absolute::LockTime::from_consensus(0),
            input: tx_ins,
            output: tx_outs,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::ledger::Ledger;
    use bitcoin::ScriptBuf;

    /// Tests transaction operations over ledger, without any rule checks.
    #[test]
    fn transactions_without_checks() {
        let ledger = Ledger::new();

        assert_eq!(ledger.get_transactions().len(), 0);

        let txout = ledger.create_txout(0x45, Some(ScriptBuf::new()));
        let tx = ledger.create_transaction(vec![], vec![txout]);
        let txid = tx.compute_txid();

        assert_eq!(
            txid,
            ledger.add_transaction_unconditionally(tx.clone()).unwrap()
        );

        let txs = ledger.get_transactions();
        assert_eq!(txs.len(), 1);

        let tx2 = txs.get(0).unwrap().to_owned();
        assert_eq!(tx, tx2);

        let tx2 = ledger.get_transaction(txid).unwrap();
        assert_eq!(tx, tx2);
    }

    /// Tests transaction operations over ledger, with rule checks.
    #[test]
    #[ignore = "Ledger under construction"]
    fn transactions_with_checks() {
        let ledger = Ledger::new();

        assert_eq!(ledger.get_transactions().len(), 0);

        let txout = ledger.create_txout(0x45 * 0x45, None);
        let tx = ledger.create_transaction(vec![], vec![txout.clone()]);
        let txid = tx.compute_txid();

        // First, add some funds to user, for free.
        assert_eq!(
            txid,
            ledger.add_transaction_unconditionally(tx.clone()).unwrap()
        );

        // Input amount is zero. This should not be accepted.
        if let Ok(_) = ledger.add_transaction(tx.clone()) {
            assert!(false);
        };

        let txin = ledger.create_txin(txid);
        let tx = ledger.create_transaction(vec![txin], vec![txout]);
        let txid = tx.compute_txid();

        // Input amount is OK. This should be accepted.
        assert_eq!(txid, ledger.add_transaction(tx.clone()).unwrap());

        let txs = ledger.get_transactions();
        assert_eq!(txs.len(), 2);

        let tx2 = txs.get(1).unwrap().to_owned();
        assert_eq!(tx, tx2);

        let tx2 = ledger.get_transaction(txid).unwrap();
        assert_eq!(tx, tx2);
    }
}
