//! # Transaction Related Ledger Operations

use super::{errors::LedgerError, Ledger};
use bitcoin::{
    absolute,
    consensus::{Decodable, Encodable},
    Amount, OutPoint, ScriptBuf, Transaction, TxIn, TxOut, Txid,
};
use rusqlite::params;

impl Ledger {
    /// Adds transaction to blockchain, after verifying.
    pub fn add_transaction(&self, transaction: Transaction) -> Result<Txid, LedgerError> {
        self.check_transaction(&transaction)?;

        self.add_transaction_unconditionally(transaction)
    }
    /// Adds transaction to blockchain, without verifying.
    pub fn add_transaction_unconditionally(
        &self,
        transaction: Transaction,
    ) -> Result<Txid, LedgerError> {
        let txid = transaction.compute_txid();

        let mut body = Vec::new();
        match transaction.consensus_encode(&mut body) {
            Ok(_) => (),
            Err(e) => return Err(LedgerError::Transaction(e.to_string())),
        };

        self.database
            .lock()
            .unwrap()
            .execute(
                "INSERT INTO \"transactions\" (txid, body) VALUES (?1, ?2)",
                params![txid.to_string(), body],
            )
            .unwrap();

        Ok(txid)
    }
    /// Returns a transaction which matches the given txid.
    pub fn get_transaction(&self, txid: Txid) -> Result<Transaction, LedgerError> {
        let tx = self
            .database
            .lock()
            .unwrap()
            .query_row(
                "SELECT body FROM transactions WHERE txid = ?1",
                params![txid.to_string()],
                |row| {
                    let body = row.get::<_, Vec<u8>>(0).unwrap();

                    let tx = Transaction::consensus_decode(&mut body.as_slice()).unwrap();

                    Ok(tx)
                },
            )
            .unwrap();

        Ok(tx)
    }
    pub fn _get_transactions(&self) -> Vec<Transaction> {
        let database = self.database.lock().unwrap();

        let mut stmt = database.prepare("SELECT body FROM transactions").unwrap();
        let tx_iter = stmt
            .query_map([], |row| {
                let body: Vec<u8> = row.get(0).unwrap();
                Ok(Transaction::consensus_decode(&mut body.as_slice()).unwrap())
            })
            .unwrap();

        let txs: Vec<Transaction> = tx_iter.map(|tx| tx.unwrap()).collect();

        txs
    }

    /// Checks if a transaction is valid or not.
    pub fn check_transaction(&self, transaction: &Transaction) -> Result<(), LedgerError> {
        let input_value = self.calculate_transaction_input_value(transaction.clone())?;
        let output_value = self.calculate_transaction_output_value(transaction.clone());

        if input_value < output_value {
            return Err(LedgerError::Transaction(format!(
                "Input amount is smaller than output amount: {} < {}",
                input_value, output_value
            )));
        }

        // TODO: Perform these checks.
        // for input in transaction.input.iter() {
        //     for input_idx in 0..transaction.input.len() {
        //         let previous_output = self.get_transaction(input.previous_output.txid)?.output;
        //         let previous_output = previous_output
        //             .get(input.previous_output.vout as usize)
        //             .unwrap()
        //             .to_owned();

        //         let script_pubkey = previous_output.clone().script_pubkey;

        //         if script_pubkey.is_p2wpkh() {
        //             let _ = P2WPKHChecker::check(&transaction, &previous_output, input_idx);
        //         } else if script_pubkey.is_p2wsh() {
        //             let _ = P2WSHChecker::check(&transaction, &previous_output, input_idx);
        //         } else if script_pubkey.is_p2tr() {
        //             let _ = P2TRChecker::check(&transaction, &previous_output, input_idx);
        //         }
        //     }
        // }

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

        for input in transaction.input {
            amount += self
                .get_transaction(input.previous_output.txid)?
                .output
                .get(input.previous_output.vout as usize)
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
    pub fn _create_txin(&self, txid: Txid, vout: u32) -> TxIn {
        TxIn {
            previous_output: OutPoint { txid, vout },
            ..Default::default()
        }
    }
    /// Creates a `TxOut` with some defaults.
    pub fn create_txout(&self, value: Amount, script_pubkey: ScriptBuf) -> TxOut {
        TxOut {
            value,
            script_pubkey,
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

        let txout = ledger.create_txout(Amount::from_sat(0x45), ScriptBuf::new());
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
        // let ledger = Ledger::new();

        // let credential = Ledger::generate_credential_from_witness();
        // ledger.add_credential(credential.clone());

        // assert_eq!(ledger.get_transactions().len(), 0);

        // // First, add some funds to user, for free.
        // let txout = ledger.create_txout(
        //     Amount::from_sat(0x45 * 0x45),
        //     credential.address.script_pubkey(),
        // );
        // let tx = ledger.create_transaction(vec![], vec![txout.clone()]);
        // let txid = tx.compute_txid();
        // assert_eq!(
        //     txid,
        //     ledger.add_transaction_unconditionally(tx.clone()).unwrap()
        // );

        // // Input amount is zero. Same transaction should not be accepted, if
        // // checks are performed..
        // if let Ok(_) = ledger.add_transaction(tx.clone()) {
        //     assert!(false);
        // };

        // // Create a valid transaction. This should pass checks.
        // let txin = ledger._create_txin(txid, 0);
        // let txout = ledger.create_txout(
        //     Amount::from_sat(0x44 * 0x45),
        //     credential.address.script_pubkey(),
        // );
        // let tx = ledger.create_transaction(vec![txin], vec![txout]);
        // let txid = tx.compute_txid();
        // assert_eq!(txid, ledger.add_transaction(tx.clone()).unwrap());

        // let txs = ledger.get_transactions();
        // assert_eq!(txs.len(), 2);

        // let read_tx = txs.get(1).unwrap().to_owned();
        // assert_eq!(tx, read_tx);

        // let read_tx = ledger.get_transaction(txid).unwrap();
        // assert_eq!(tx, read_tx);
    }

    #[test]
    fn calculate_transaction_input_value() {
        let ledger = Ledger::new();

        let address = Ledger::_generate_address_from_witness();

        // Add some funds.
        let txout = ledger.create_txout(Amount::from_sat(0x45), address.script_pubkey());
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
        let txin = ledger._create_txin(txid, 0);
        let tx = ledger.create_transaction(vec![txin], vec![txout]);
        assert_eq!(
            ledger.calculate_transaction_input_value(tx).unwrap(),
            Amount::from_sat(0x45)
        );
    }

    #[test]
    fn calculate_transaction_output_value() {
        let ledger = Ledger::new();

        let txout1 = ledger.create_txout(Amount::from_sat(0x45), ScriptBuf::new());
        let tx = ledger.create_transaction(vec![], vec![txout1.clone()]);
        assert_eq!(
            ledger.calculate_transaction_output_value(tx),
            Amount::from_sat(0x45)
        );

        let txout2 = ledger.create_txout(Amount::from_sat(0x1F), ScriptBuf::new());
        let tx = ledger.create_transaction(vec![], vec![txout1, txout2]);
        assert_eq!(
            ledger.calculate_transaction_output_value(tx),
            Amount::from_sat(100)
        );
    }
}
