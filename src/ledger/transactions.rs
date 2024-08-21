//! # Transaction Related Ledger Operations

use super::{errors::LedgerError, spending_requirements::SpendingRequirementsReturn, Ledger};
use crate::utils::{self, hex_to_array, Hash256};
use bitcoin::{
    absolute::{self, LockTime},
    consensus::{
        encode::{deserialize_hex, serialize_hex},
        Decodable, Encodable,
    },
    hashes::{sha256d, Hash},
    opcodes::all::OP_RETURN,
    Address, Amount, BlockHash, OutPoint, ScriptBuf, Sequence, Transaction, TxIn, TxMerkleNode,
    TxOut, Txid, Witness, Wtxid,
};
use bitcoin_scriptexec::{ExecCtx, TxTemplate};
use rs_merkle::Hasher;
use rusqlite::params;

impl Ledger {
    /// Adds transaction to blockchain, after verifying.
    #[tracing::instrument]
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

        tracing::trace!(
            "Adding new transaction {transaction:?} at block height {current_block_height}"
        );

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
        tracing::trace!("Fetching transaction with txid {txid:?}");

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
        tracing::trace!("Getting block height for transaction with txid {txid:?}");

        let block_height = self.database.lock().unwrap().query_row(
            "SELECT block_height FROM transactions WHERE txid = ?1",
            params![txid.to_string()],
            |row| {
                let body = row.get::<_, u32>(0).unwrap();

                Ok(body)
            },
        );
        let block_height = match block_height {
            Ok(block_height) => block_height,
            Err(e) => {
                return Err(LedgerError::Transaction(format!(
                    "Couldn't get block height for txid {}: {}",
                    txid, e
                )))
            }
        };

        Ok(block_height)
    }

    pub fn get_transaction_block_hash(&self, txid: &Txid) -> Result<BlockHash, LedgerError> {
        tracing::trace!("Getting block hash for transaction with txid {txid:?}");

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

    pub fn _get_transactions(&self) -> Vec<Transaction> {
        tracing::trace!("Fetching all the transactions");

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
    #[tracing::instrument]
    pub fn check_transaction(&self, transaction: &Transaction) -> Result<(), LedgerError> {
        self.check_transaction_funds(transaction)?;

        let mut txouts = vec![];
        for input in transaction.input.iter() {
            if input.script_sig.len() != 0 {
                let msg = "Bitcoin mock RPC only verifies inputs that support segregated witness.";
                tracing::error!(msg);

                return Err(LedgerError::Transaction(msg.to_string()));
            }

            let tx = self.get_transaction(input.previous_output.txid)?;
            let txout = tx
                .output
                .get(input.previous_output.vout as usize)
                .unwrap()
                .to_owned();

            txouts.push(txout);
        }
        tracing::trace!("UTXOs that will be spent in this transaction: {txouts:?}");

        for input_idx in 0..transaction.input.len() {
            let mut ret: SpendingRequirementsReturn = SpendingRequirementsReturn::default();
            let mut ctx: ExecCtx = ExecCtx::Legacy;

            if txouts[input_idx].script_pubkey.is_p2wpkh() {
                tracing::trace!("Input with index {input_idx} is a P2WPKH");
                self.p2wpkh_check(transaction, txouts.as_slice(), input_idx)?;
                continue;
            } else if txouts[input_idx].script_pubkey.is_p2wsh() {
                tracing::trace!("Input with index {input_idx} is a P2WSH");
                ret = self.p2wsh_check(transaction, &txouts, input_idx)?;
                ctx = ExecCtx::SegwitV0;
            } else if txouts[input_idx].script_pubkey.is_p2tr() {
                tracing::trace!("Input with index {input_idx} is a P2TR");
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

    /// Checks if transactions input amount is equal or bigger than the output
    /// amount.
    pub fn check_transaction_funds(&self, transaction: &Transaction) -> Result<(), LedgerError> {
        let input_value = self.calculate_transaction_input_value(transaction)?;
        let output_value = self.calculate_transaction_output_value(transaction);

        if input_value < output_value {
            Err(LedgerError::Transaction(format!(
                "Input amount is smaller than output amount: {} < {}",
                input_value, output_value
            )))
        } else {
            Ok(())
        }
    }

    /// Calculates a transaction's total output value.
    ///
    /// # Panics
    ///
    /// Panics if found UTXO doesn't match transaction.
    pub fn calculate_transaction_input_value(
        &self,
        transaction: &Transaction,
    ) -> Result<Amount, LedgerError> {
        let mut amount = Amount::from_sat(0);

        for input in &transaction.input {
            amount += self
                .get_transaction(input.previous_output.txid)?
                .output
                .get(input.previous_output.vout as usize)
                .unwrap()
                .value;
        }

        tracing::trace!("Transaction's input value in total is {amount}");

        Ok(amount)
    }

    /// Calculates a transaction's total output value.
    pub fn calculate_transaction_output_value(&self, transaction: &Transaction) -> Amount {
        let amount = transaction.output.iter().map(|output| output.value).sum();

        tracing::trace!("Transaction's output value in total is {amount}");

        amount
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

    /// Creates the special coinbase transaction.
    ///
    /// # Parameters
    ///
    /// - address: Miner's address
    /// - wtxid_merkle_root: Merkle root of all the transaction wTXID's
    pub fn create_coinbase_transaction(
        &self,
        address: &Address,
        wtxids: Vec<Wtxid>,
    ) -> Result<Transaction, LedgerError> {
        tracing::trace!("Creating coinbase transaction for address {address:?}");

        let current_block_height = self.get_block_height()? + 1;
        let mut script_sig = ScriptBuf::new();
        script_sig.push_slice(current_block_height.to_be_bytes());
        tracing::trace!("Input script sig {script_sig:?}");

        let mut witness = Witness::new();
        witness.push([0u8; 32]);
        tracing::trace!("Input witness {witness:?}");

        // Insert all zeroed wTXID to the list (coinbase transaction).
        let mut wtxids = wtxids.clone();
        wtxids.insert(0, Wtxid::all_zeros());
        tracing::trace!("Final wTXIDs: {wtxids:?}");

        // Calculate merkle root of input wTXIDs.
        let merkle_root = utils::calculate_merkle_root(wtxids)?;
        tracing::trace!("Merkle root of the wTXIDs: {merkle_root:?}");

        // Prepare wTXID commitment.
        let concat = serialize_hex::<TxMerkleNode>(&merkle_root)
            + "0000000000000000000000000000000000000000000000000000000000000000";
        let mut hex: [u8; 64] = [0; 64];
        hex_to_array(&concat, &mut hex);
        let wtxid_commitment = Hash256::hash(hex.as_slice());
        tracing::trace!("wTXID commitment: {:?}", serialize_hex(&wtxid_commitment));

        // Assign wTXID commitment header.
        let header = deserialize_hex::<[u8; 4]>("aa21a9ed").unwrap();
        let mut hex: [u8; 36] = [0u8; 36];
        header.iter().enumerate().for_each(|(idx, char)| {
            hex[idx] = *char;
        });

        // Assign wTXID commitment.
        wtxid_commitment.iter().enumerate().for_each(|(idx, char)| {
            hex[idx + 4] = *char;
        });

        // Prepare script pubkey.
        let mut script_pubkey = ScriptBuf::new();
        script_pubkey.push_opcode(OP_RETURN);
        script_pubkey.push_slice(hex);
        tracing::trace!("Output script pubkey: {:?}", script_pubkey);

        Ok(Transaction {
            version: bitcoin::transaction::Version::TWO,
            lock_time: LockTime::ZERO,
            input: vec![TxIn {
                previous_output: OutPoint {
                    txid: Txid::all_zeros(),
                    vout: u32::MAX,
                },
                script_sig,
                sequence: Sequence::ZERO,
                witness,
            }],
            output: vec![
                TxOut {
                    value: Amount::from_sat(crate::utils::BLOCK_REWARD),
                    script_pubkey: address.script_pubkey(),
                },
                TxOut {
                    value: Amount::from_sat(0),
                    script_pubkey,
                },
            ],
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        ledger::{self, Ledger},
        utils::hex_to_array,
    };
    use bitcoin::{
        hashes::Hash, opcodes::all::OP_RETURN, Amount, OutPoint, ScriptBuf, TxIn, Txid, Wtxid,
    };
    use std::str::FromStr;

    /// Tests transaction operations over ledger, without any rule checks.
    #[test]
    fn transactions_without_checks() {
        let ledger = Ledger::new("transactions_without_checks");

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
        let ledger = Ledger::new("transactions_with_checks");

        let credentials = Ledger::generate_credential_from_witness();
        let address = credentials.address;

        assert_eq!(ledger._get_transactions().len(), 0);

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

        let txs = ledger._get_transactions();
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
            ledger.calculate_transaction_input_value(&tx).unwrap(),
            Amount::from_sat(0)
        );
        // Valid input should be OK.
        let txin = ledger.create_txin(txid, 0);
        let tx = ledger.create_transaction(vec![txin], vec![txout]);
        assert_eq!(
            ledger.calculate_transaction_input_value(&tx).unwrap(),
            Amount::from_sat(0x45)
        );
    }

    #[test]
    fn calculate_transaction_output_value() {
        let ledger = Ledger::new("calculate_transaction_output_value");

        let txout1 = ledger.create_txout(Amount::from_sat(0x45), ScriptBuf::new());
        let tx = ledger.create_transaction(vec![], vec![txout1.clone()]);
        assert_eq!(
            ledger.calculate_transaction_output_value(&tx),
            Amount::from_sat(0x45)
        );

        let txout2 = ledger.create_txout(Amount::from_sat(0x1F), ScriptBuf::new());
        let tx = ledger.create_transaction(vec![], vec![txout1, txout2]);
        assert_eq!(
            ledger.calculate_transaction_output_value(&tx),
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

    #[test]
    fn create_coinbase_transaction() {
        let ledger = Ledger::new("create_coinbase_transaction");

        let address = ledger::Ledger::generate_credential().address;
        let wtxids: Vec<Wtxid> = vec![
            Wtxid::from_str("8700d546b39e1a0faf34c98067356206db50fdef24e2f70b431006c59d548ea2")
                .unwrap(),
            Wtxid::from_str("c54bab5960d3a416c40464fa67af1ddeb63a2ce60a0b3c36f11896ef26cbcb87")
                .unwrap(),
            Wtxid::from_str("e51de361009ef955f182922647622f9662d1a77ca87c4eb2fd7996b2fe0d7785")
                .unwrap(),
        ];

        let tx = ledger
            .create_coinbase_transaction(&address, wtxids)
            .unwrap();

        assert_eq!(tx.input.len(), 1);
        assert_eq!(
            tx.input.first().unwrap().previous_output,
            OutPoint {
                txid: Txid::all_zeros(),
                vout: u32::MAX
            }
        );

        let expected_script_pubkey = {
            let mut hex: [u8; 36] = [0; 36];
            hex_to_array(
                "aa21a9ed6502e8637ba29cd8a820021915339c7341223d571e5e8d66edd83786d387e715",
                &mut hex,
            );

            let mut expected_script_pubkey = ScriptBuf::new();
            expected_script_pubkey.push_opcode(OP_RETURN);
            expected_script_pubkey.push_slice(hex);

            expected_script_pubkey
        };
        assert_eq!(
            tx.output.get(1).unwrap().script_pubkey,
            expected_script_pubkey
        );
    }
}
