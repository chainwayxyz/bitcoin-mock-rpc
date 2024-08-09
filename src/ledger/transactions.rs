//! # Transaction Related Ledger Operations

use super::{errors::LedgerError, spending_requirements::SpendingRequirementsReturn, Ledger};
use bitcoin::{
    absolute,
    consensus::{Decodable, Encodable},
    hashes::sha256d,
    Amount, BlockHash, OutPoint, ScriptBuf, Transaction, TxIn, TxOut, Txid,
};
use bitcoin_scriptexec::{ExecCtx, TxTemplate};
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
        let current_block_height = self.get_block_height()?;

        let mut body = Vec::new();
        match transaction.consensus_encode(&mut body) {
            Ok(_) => (),
            Err(e) => return Err(LedgerError::Transaction(e.to_string())),
        };

        // Use next block height as the transaction height.
        if let Err(e) = self.database.lock().unwrap().execute(
            "INSERT INTO transactions (txid, block_height, body) VALUES (?1, ?2, ?3)",
            params![txid.to_string(), current_block_height + 1, body],
        ) {
            return Err(LedgerError::Transaction(format!(
                "Couldn't add transaction with txid {} to ledger: {}",
                txid, e
            )));
        };

        self.add_mempool_transaction(txid)?;

        Ok(txid)
    }

    /// Returns a transaction which matches the given txid.
    pub fn get_transaction(&self, txid: Txid) -> Result<Transaction, LedgerError> {
        let tx = self.database.lock().unwrap().query_row(
            "SELECT body FROM transactions WHERE txid = ?1",
            params![txid.to_string()],
            |row| {
                let body = row.get::<_, Vec<u8>>(0).unwrap();

                let tx = Transaction::consensus_decode(&mut body.as_slice()).unwrap();

                Ok(tx)
            },
        );
        let tx = match tx {
            Ok(tx) => tx,
            Err(e) => {
                return Err(LedgerError::Transaction(format!(
                    "Couldn't found transaction with txid {}: {}",
                    txid, e
                )))
            }
        };

        Ok(tx)
    }

    pub fn get_transaction_block_height(&self, txid: &Txid) -> Result<u32, LedgerError> {
        let sequence = self.database.lock().unwrap().query_row(
            "SELECT block_height FROM transactions WHERE txid = ?1",
            params![txid.to_string()],
            |row| {
                let body = row.get::<_, u32>(0).unwrap();

                Ok(body)
            },
        );
        let sequence = match sequence {
            Ok(sequence) => sequence,
            Err(e) => {
                return Err(LedgerError::Transaction(format!(
                    "Couldn't get block height for txid {}: {}",
                    txid, e
                )))
            }
        };

        Ok(sequence)
    }

    pub fn get_transaction_block_hash(&self, txid: &Txid) -> Result<BlockHash, LedgerError> {
        let height = self.get_transaction_block_height(txid)?;

        let hash = self.database.lock().unwrap().query_row(
            "SELECT hash FROM blocks WHERE height = ?1",
            params![height],
            |row| row.get::<_, Vec<u8>>(0),
        );

        let hash = match hash {
            Ok(h) => {
                let hash = sha256d::Hash::consensus_decode(&mut h.as_slice()).unwrap();
                BlockHash::from_raw_hash(hash)
            }
            Err(_) => return Err(LedgerError::BlockInMempool(height)),
        };

        Ok(hash)
    }

    pub fn get_transactions(&self) -> Vec<Transaction> {
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

    /// Checks if a transaction is valid or not. Steps:
    ///
    /// 1. Is input value is larger than the output value?
    /// 2. Is satisfies it's spending requirements?
    /// 3. Is script execution successful?
    ///
    /// No checks for if that UTXO is spendable or not.
    pub fn check_transaction(&self, transaction: &Transaction) -> Result<(), LedgerError> {
        let input_value = self.calculate_transaction_input_value(transaction.clone())?;
        let output_value = self.calculate_transaction_output_value(transaction.clone());

        if input_value < output_value {
            return Err(LedgerError::Transaction(format!(
                "Input amount is smaller than output amount: {} < {}",
                input_value, output_value
            )));
        }

        let mut prev_outs = vec![];
        let mut txouts = vec![];
        for input in transaction.input.iter() {
            assert_eq!(
                input.script_sig.len(),
                0,
                "Bitcoin simulator only verifies inputs that support segregated witness."
            );

            let tx = self.get_transaction(input.previous_output.txid)?;
            let txout = tx
                .output
                .get(input.previous_output.vout as usize)
                .unwrap()
                .to_owned();

            txouts.push(txout);
            prev_outs.push(input.previous_output);
        }

        for input_idx in 0..transaction.input.len() {
            let mut ret: SpendingRequirementsReturn = SpendingRequirementsReturn::default();
            let mut ctx: ExecCtx = ExecCtx::Legacy;

            if txouts[input_idx].script_pubkey.is_p2wpkh() {
                self.p2wpkh_check(transaction, txouts.as_slice(), input_idx)?;
                continue;
            } else if txouts[input_idx].script_pubkey.is_p2wsh() {
                ret = self.p2wsh_check(transaction, &txouts, input_idx)?;
                ctx = ExecCtx::SegwitV0;
            } else if txouts[input_idx].script_pubkey.is_p2tr() {
                ret = self.p2tr_check(transaction, &txouts, input_idx)?;
                if ret.taproot.is_none() {
                    continue;
                }
                ctx = ExecCtx::Tapscript;
            }

            let tx_template = TxTemplate {
                tx: transaction.clone(),
                prevouts: txouts.to_vec(),
                input_idx,
                taproot_annex_scriptleaf: ret.taproot,
            };

            self.run_script(ctx, tx_template, ret.script_buf, ret.witness)?;
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
    pub fn create_txin(&self, txid: Txid, vout: u32) -> TxIn {
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
    use bitcoin::{hashes::Hash, Amount, OutPoint, ScriptBuf, TxIn, Txid};

    /// Tests transaction operations over ledger, without any rule checks.
    #[test]
    fn transactions_without_checks() {
        let ledger = Ledger::new("transactions_without_checks");

        assert_eq!(ledger.get_transactions().len(), 0);

        let txout = ledger.create_txout(Amount::from_sat(0x45), ScriptBuf::new());
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
    fn transactions_with_checks() {
        let ledger = Ledger::new("transactions_with_checks");

        let credentials = Ledger::generate_credential_from_witness();
        let address = credentials.address;

        assert_eq!(ledger.get_transactions().len(), 0);

        // First, add some funds to user, for free.
        let txout = ledger.create_txout(Amount::from_sat(0x45 * 0x45), address.script_pubkey());
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
        let txin = TxIn {
            previous_output: OutPoint { txid, vout: 0 },
            witness: credentials.witness.unwrap(),
            ..Default::default()
        };
        let txout = ledger.create_txout(Amount::from_sat(0x44 * 0x45), address.script_pubkey());
        let tx = ledger.create_transaction(vec![txin], vec![txout]);
        let txid = tx.compute_txid();
        assert_eq!(txid, ledger.add_transaction(tx.clone()).unwrap());

        let txs = ledger.get_transactions();
        assert_eq!(txs.len(), 2);

        let read_tx = txs.get(1).unwrap().to_owned();
        assert_eq!(tx, read_tx);

        let read_tx = ledger.get_transaction(txid).unwrap();
        assert_eq!(tx, read_tx);
    }

    #[test]
    fn calculate_transaction_input_value() {
        let ledger = Ledger::new("calculate_transaction_input_value");

        let address = Ledger::generate_address_from_witness();

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
        let txin = ledger.create_txin(txid, 0);
        let tx = ledger.create_transaction(vec![txin], vec![txout]);
        assert_eq!(
            ledger.calculate_transaction_input_value(tx).unwrap(),
            Amount::from_sat(0x45)
        );
    }

    #[test]
    fn calculate_transaction_output_value() {
        let ledger = Ledger::new("calculate_transaction_output_value");

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

    #[test]
    #[should_panic]
    fn check_transaction_wiht_low_input_value() {
        let ledger = Ledger::new("check_transaction_wiht_low_input_value");

        let txout = ledger.create_txout(Amount::from_sat(0x45), ScriptBuf::new());
        let tx = ledger.create_transaction(vec![], vec![txout.clone()]);
        let txid = ledger.add_transaction_unconditionally(tx).unwrap();

        let txin = ledger.create_txin(txid, 0);
        let txout = ledger.create_txout(Amount::from_sat(0x100), ScriptBuf::new());
        let tx = ledger.create_transaction(vec![txin], vec![txout.clone()]);
        ledger.check_transaction(&tx).unwrap();
    }

    #[test]
    #[should_panic]
    fn get_transaction_non_existing() {
        let ledger = Ledger::new("get_transaction_non_existing");
        ledger.get_transaction(Txid::all_zeros()).unwrap();
    }

    #[test]
    #[should_panic]
    fn get_transaction_block_height_non_existing() {
        let ledger = Ledger::new("get_transaction_block_height_non_existing");
        ledger
            .get_transaction_block_height(&Txid::all_zeros())
            .unwrap();
    }
}
