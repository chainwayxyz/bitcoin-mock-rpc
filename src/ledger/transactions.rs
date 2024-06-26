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
        let txid = transaction.compute_txid();

        // Remove UTXO's that are used.
        transaction.input.iter().for_each(|input| {
            self.remove_utxo(input.previous_output);
        });

        // Add UTXO's that are sent to user.
        let script_pubkeys: Vec<ScriptBuf> = self
            .get_credentials()
            .iter()
            .map(|credential| credential.address.script_pubkey())
            .collect();
        transaction.output.iter().enumerate().for_each(|(i, utxo)| {
            if script_pubkeys
                .iter()
                .any(|hash| *hash == utxo.script_pubkey)
            {
                let utxo = OutPoint {
                    txid,
                    vout: i as u32,
                };
                self.add_utxo(utxo);
            }
        });

        // Add transaction to list.
        add_item_to_vec!(self.transactions, transaction.clone());

        Ok(txid)
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
    pub fn _get_transactions(&self) -> Vec<Transaction> {
        return_vec_item!(self.transactions);
    }

    /// Checks if a transaction is valid or not.
    pub fn check_transaction(&self, transaction: &Transaction) -> Result<(), LedgerError> {
        let input_value = self.calculate_transaction_input_value(transaction.clone())?;
        let output_value = self.calculate_transaction_output_value(transaction.clone());

        if input_value < output_value {
            return Err(LedgerError::Transaction(format!(
                "Input value {} is not above or equal of output value {}",
                input_value, output_value
            )));
        }

        Ok(())
    }

    /// Calculates a transaction's total output value.
    ///
    /// # Panics
    ///
    /// Panics if found UTXO doesn't match transaction.
    pub fn calculate_transaction_input_value(
        &self,
        transaction: Transaction,
    ) -> Result<Amount, LedgerError> {
        let mut amount = Amount::from_sat(0);
        let utxos = self.get_utxos();

        for input in transaction.input {
            let utxo = utxos
                .iter()
                .find(|utxo| **utxo == input.previous_output)
                .ok_or(LedgerError::UTXO(format!(
                    "UTXO {:?} is not found in UTXO list.",
                    input.previous_output
                )))?;

            amount += self
                .get_transaction(utxo.txid)?
                .output
                .get(utxo.vout as usize)
                .unwrap()
                .value;
        }

        Ok(amount)
    }
    /// Calculates a transaction's total output value.
    pub fn calculate_transaction_output_value(&self, transaction: Transaction) -> Amount {
        transaction.output.iter().map(|output| output.value).sum()
    }

    /// Creates a `TxIn` with some defaults.
    pub fn create_txin(&self, txid: Txid, vout: u32) -> TxIn {
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
            previous_output: OutPoint { txid, vout },
            witness,
            ..Default::default()
        }
    }
    /// Creates a `TxOut` with some defaults.
    pub fn create_txout(&self, amount: Amount, script_pubkey: Option<ScriptBuf>) -> TxOut {
        TxOut {
            value: amount,
            script_pubkey: match script_pubkey {
                Some(script_pubkey) => script_pubkey,
                None => ScriptBuf::new(),
            },
        }
    }
    /// Creates a `Transaction` with some defaults.
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
    use bitcoin::{Amount, ScriptBuf};

    /// Tests transaction operations over ledger, without any rule checks.
    #[test]
    fn transactions_without_checks() {
        let ledger = Ledger::new();

        assert_eq!(ledger._get_transactions().len(), 0);

        let txout = ledger.create_txout(Amount::from_sat(0x45), Some(ScriptBuf::new()));
        let tx = ledger.create_transaction(vec![], vec![txout]);
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
        let credentials = ledger.generate_credential();

        assert_eq!(ledger._get_transactions().len(), 0);

        // First, add some funds to user, for free.
        let txout = ledger.create_txout(
            Amount::from_sat(0x45 * 0x45),
            Some(credentials.address.script_pubkey()),
        );
        let tx = ledger.create_transaction(vec![], vec![txout.clone()]);
        let txid = tx.compute_txid();
        assert_eq!(
            txid,
            ledger.add_transaction_unconditionally(tx.clone()).unwrap()
        );

        // Input amount is zero. Same transaction should not be accepted, if
        // checks are performed..
        if let Ok(_) = ledger.add_transaction(tx.clone()) {
            assert!(false);
        };

        // Create a valid transaction. This should pass checks.
        let txin = ledger.create_txin(txid, 0);
        let txout = ledger.create_txout(Amount::from_sat(0x44 * 0x45), None);
        let tx = ledger.create_transaction(vec![txin], vec![txout]);
        let txid = tx.compute_txid();
        assert_eq!(txid, ledger.add_transaction(tx.clone()).unwrap());

        let txs = ledger._get_transactions();
        assert_eq!(txs.len(), 2);

        let read_tx = txs.get(1).unwrap().to_owned();
        assert_eq!(tx, read_tx);

        let read_tx = ledger.get_transaction(txid).unwrap();
        assert_eq!(tx, read_tx);
    }

    #[test]
    fn calculate_transaction_input_value() {
        let ledger = Ledger::new();
        let credential = ledger.generate_credential();

        // Add some funds.
        let txout = ledger.create_txout(
            Amount::from_sat(0x45),
            Some(credential.address.script_pubkey()),
        );
        let tx = ledger.create_transaction(vec![], vec![txout.clone()]);
        let txid = tx.compute_txid();
        assert_eq!(
            txid,
            ledger.add_transaction_unconditionally(tx.clone()).unwrap()
        );

        // Without any inputs, this must return 0 Sats.
        assert_eq!(
            ledger.calculate_transaction_input_value(tx).unwrap(),
            Amount::from_sat(0)
        );
        // Valid input should be OK.
        let txin = ledger.create_txin(txid, 0);
        let tx = ledger.create_transaction(vec![txin], vec![txout]);
        assert_eq!(
            ledger.calculate_transaction_input_value(tx).unwrap(),
            Amount::from_sat(0x45)
        );
    }

    #[test]
    fn calculate_transaction_output_value() {
        let ledger = Ledger::new();

        let txout1 = ledger.create_txout(Amount::from_sat(0x45), None);
        let tx = ledger.create_transaction(vec![], vec![txout1.clone()]);
        assert_eq!(
            ledger.calculate_transaction_output_value(tx),
            Amount::from_sat(0x45)
        );

        let txout2 = ledger.create_txout(Amount::from_sat(0x1F), None);
        let tx = ledger.create_transaction(vec![], vec![txout1, txout2]);
        assert_eq!(
            ledger.calculate_transaction_output_value(tx),
            Amount::from_sat(100)
        );
    }
}
