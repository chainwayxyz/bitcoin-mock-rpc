//! # RPC API
//!
//! This crate implements `RpcApi` trait in `bitcoincore-rpc` for the mock
//! `Client`.

use super::Client;
use crate::{
    ledger::{self, errors::LedgerError},
    utils::encode_to_hex,
};
use bitcoin::{
    address::NetworkChecked,
    consensus::{encode, serialize, Encodable},
    hashes::Hash,
    params::Params,
    Address, Amount, BlockHash, OutPoint, SignedAmount, Transaction, TxIn, TxOut, Txid,
};
use bitcoincore_rpc::{
    json::{
        self, GetChainTipsResultStatus, GetRawTransactionResult, GetRawTransactionResultVin,
        GetRawTransactionResultVinScriptSig, GetRawTransactionResultVout,
        GetRawTransactionResultVoutScriptPubKey, GetTransactionResult, GetTransactionResultDetail,
        GetTransactionResultDetailCategory, GetTxOutResult, SignRawTransactionResult, WalletTxInfo,
    },
    Error, RpcApi,
};
use secp256k1::rand::{self, RngCore};

impl RpcApi for Client {
    /// TL;DR: If this function is called for `cmd`, it's corresponding mock is
    /// not yet implemented. Please consider implementing it. Ellerinden oper
    /// diyorum anlamadiysan.
    ///
    /// This function normally talks with Bitcoin network. Therefore, other
    /// functions calls this to send requests. In a mock environment though,
    /// other functions won't be talking to a regulated interface. Rather will
    /// access a temporary in-memory database.
    ///
    /// This is the reason, this function will only throw errors in case of a
    /// function calls this. Tester should implement corresponding function in
    /// this impl block.
    #[tracing::instrument(skip_all)]
    fn call<T: for<'a> serde::de::Deserialize<'a>>(
        &self,
        cmd: &str,
        args: &[serde_json::Value],
    ) -> bitcoincore_rpc::Result<T> {
        let msg = format!(
            "Unimplemented mock RPC cmd: {}, with args: {:?}. Please consider implementing it.",
            cmd, args
        );

        tracing::error!(msg);

        Err(Error::ReturnedError(msg))
    }

    #[tracing::instrument(skip_all)]
    fn send_raw_transaction<R: bitcoincore_rpc::RawTx>(
        &self,
        tx: R,
    ) -> bitcoincore_rpc::Result<bitcoin::Txid> {
        let tx: Transaction = encode::deserialize_hex(&tx.raw_hex())?;

        self.ledger.add_transaction(tx.clone())?;

        Ok(tx.compute_txid())
    }
    #[tracing::instrument(skip_all)]
    fn get_raw_transaction(
        &self,
        txid: &bitcoin::Txid,
        _block_hash: Option<&bitcoin::BlockHash>,
    ) -> bitcoincore_rpc::Result<bitcoin::Transaction> {
        if _block_hash.is_some() {
            return Err(Error::ReturnedError(format!(
                "This argument is unimplemented: {}",
                stringify!(_block_hash)
            )));
        }

        Ok(self.ledger.get_transaction(*txid)?)
    }
    /// Verbose flag enabled `get_raw_transaction`.
    ///
    /// This function **is not** intended to return information about the
    /// transaction itself. Instead, it mostly provides information about the
    /// transaction's state in blockchain. It is recommmended to use
    /// `get_raw_transaction` for information about transaction's inputs and
    /// outputs.
    #[tracing::instrument(skip_all)]
    fn get_raw_transaction_info(
        &self,
        txid: &bitcoin::Txid,
        _block_hash: Option<&bitcoin::BlockHash>,
    ) -> bitcoincore_rpc::Result<json::GetRawTransactionResult> {
        if _block_hash.is_some() {
            return Err(Error::ReturnedError(format!(
                "This argument is unimplemented: {}",
                stringify!(_block_hash)
            )));
        }

        let tx = self.get_raw_transaction(txid, _block_hash)?;

        let mut hex: Vec<u8> = Vec::new();
        if tx.consensus_encode(&mut hex).is_err() {
            hex = vec![];
        };

        let vin: Vec<GetRawTransactionResultVin> = tx
            .input
            .iter()
            .map(|input| {
                let mut txid: Option<Txid> = None;
                let mut sequence = 0;
                let mut vout: Option<u32> = None;
                let mut script_sig: Option<GetRawTransactionResultVinScriptSig> = None;
                let mut txinwitness: Option<Vec<Vec<u8>>> = None;

                if let Ok(input_tx) = self.ledger.get_transaction(input.previous_output.txid) {
                    txid = Some(input_tx.compute_txid());
                    sequence = 0;
                    vout = Some(0);
                    script_sig = None;
                    txinwitness = None;
                };

                GetRawTransactionResultVin {
                    sequence,
                    coinbase: None,
                    txid,
                    vout,
                    script_sig,
                    txinwitness,
                }
            })
            .collect();

        let vout: Vec<GetRawTransactionResultVout> = tx
            .output
            .iter()
            .enumerate()
            .map(|(idx, output)| {
                let script_pub_key = GetRawTransactionResultVoutScriptPubKey {
                    asm: "".to_string(),
                    hex: vec![],
                    req_sigs: None,
                    type_: None,
                    addresses: vec![],
                    address: None,
                };

                GetRawTransactionResultVout {
                    value: output.value,
                    n: idx as u32,
                    script_pub_key,
                }
            })
            .collect();

        let current_block_height = self.ledger.get_block_height()?;
        let tx_block_height = self
            .ledger
            .get_transaction_block_height(&tx.compute_txid())?;
        let blockhash = match self.ledger.get_transaction_block_hash(txid) {
            Ok(bh) => Some(bh),
            Err(_) => None,
        };
        let blocktime = match self.ledger.get_block_time(tx_block_height) {
            Ok(bt) => Some(bt as usize),
            Err(_) => None,
        };
        let confirmations = match self.ledger.get_mempool_transaction(*txid) {
            Some(_) => None,
            None => Some(current_block_height - tx_block_height + 1),
        };

        Ok(GetRawTransactionResult {
            in_active_chain: Some(true),
            hex,
            txid: *txid,
            hash: tx.compute_wtxid(),
            size: tx.base_size(),
            vsize: tx.vsize(),
            version: tx.version.0 as u32,
            locktime: 0,
            vin,
            vout,
            blockhash,
            confirmations,
            time: None,
            blocktime,
        })
    }

    #[tracing::instrument(skip_all)]
    fn get_transaction(
        &self,
        txid: &bitcoin::Txid,
        _include_watchonly: Option<bool>,
    ) -> bitcoincore_rpc::Result<json::GetTransactionResult> {
        if _include_watchonly.is_some() {
            return Err(Error::ReturnedError(format!(
                "This argument is unimplemented: {}",
                stringify!(_include_watchonly)
            )));
        }

        let raw_tx = self.get_raw_transaction(txid, None).unwrap();
        let mut amount = Amount::from_sat(0);

        let details: Vec<GetTransactionResultDetail> = raw_tx
            .output
            .iter()
            .map(|output| {
                amount += output.value;
                let address = match Address::from_script(
                    &output.script_pubkey,
                    Params::new(bitcoin::Network::Regtest),
                ) {
                    Ok(a) => Some(a.as_unchecked().clone()),
                    Err(_) => None,
                };

                GetTransactionResultDetail {
                    address,
                    category: GetTransactionResultDetailCategory::Send,
                    amount: SignedAmount::from_sat(output.value.to_sat() as i64),
                    label: None,
                    vout: 0,
                    fee: None,
                    abandoned: None,
                }
            })
            .collect();

        let current_height = self.ledger.get_block_height()?;
        let current_time = self.ledger.get_block_time(current_height)?;
        let tx_block_height = self.ledger.get_transaction_block_height(txid)?;
        let tx_block_time = self.ledger.get_block_time(tx_block_height)?;
        let blockhash = match self.ledger.get_transaction_block_hash(txid) {
            Ok(h) => Some(h),
            Err(_) => None,
        };
        let info = WalletTxInfo {
            confirmations: (current_height as i64 - tx_block_height as i64 + 1) as i32,
            blockhash,
            blockindex: None,
            blocktime: Some(current_time as u64),
            blockheight: Some(current_height),
            txid: *txid,
            time: current_time as u64,
            timereceived: tx_block_time as u64,
            bip125_replaceable: json::Bip125Replaceable::Unknown,
            wallet_conflicts: vec![],
        };

        Ok(GetTransactionResult {
            info,
            amount: SignedAmount::from_sat(amount.to_sat() as i64),
            fee: None,
            details,
            hex: encode::serialize(&raw_tx),
        })
    }

    /// Sends specified amount to `address` regardless of the user balance.
    /// Meaning: Unlimited free money.
    ///
    /// Reason this call behaves like this is there are no wallet
    /// implementation. This is intended way to generate inputs for other
    /// transactions.
    #[tracing::instrument(skip_all)]
    fn send_to_address(
        &self,
        address: &Address<NetworkChecked>,
        amount: Amount,
        _comment: Option<&str>,
        _comment_to: Option<&str>,
        _subtract_fee: Option<bool>,
        _replaceable: Option<bool>,
        _confirmation_target: Option<u32>,
        _estimate_mode: Option<json::EstimateMode>,
    ) -> bitcoincore_rpc::Result<bitcoin::Txid> {
        if _comment.is_some() {
            return Err(Error::ReturnedError(format!(
                "This argument is unimplemented: {}",
                stringify!(_comment)
            )));
        }
        if _comment_to.is_some() {
            return Err(Error::ReturnedError(format!(
                "This argument is unimplemented: {}",
                stringify!(_comment_to)
            )));
        }
        if _subtract_fee.is_some() {
            return Err(Error::ReturnedError(format!(
                "This argument is unimplemented: {}",
                stringify!(_subtract_fee)
            )));
        }
        if _replaceable.is_some() {
            return Err(Error::ReturnedError(format!(
                "This argument is unimplemented: {}",
                stringify!(_replaceable)
            )));
        }
        if _confirmation_target.is_some() {
            return Err(Error::ReturnedError(format!(
                "This argument is unimplemented: {}",
                stringify!(_confirmation_target)
            )));
        }
        if _estimate_mode.is_some() {
            return Err(Error::ReturnedError(format!(
                "This argument is unimplemented: {}",
                stringify!(_estimate_mode)
            )));
        }

        // First, create a random input. Why? Because calling this function for
        // same amount twice will trigger a database error about same TXID blah,
        // blah, blah.
        let rn = rand::thread_rng().next_u64();
        let txin = self.ledger.create_txin(
            Txid::hash(&[(rn & 0xFF) as u8]),
            (rn & (u32::MAX as u64)) as u32,
        );

        let txout = self.ledger.create_txout(amount, address.script_pubkey());
        let tx = self.ledger.create_transaction(vec![txin], vec![txout]);

        Ok(self.ledger.add_transaction_unconditionally(tx)?)
    }

    /// Creates a random secret/public key pair and generates a Bitcoin address
    /// from witness program. Please note that this address is not hold in
    /// ledger in any way.
    #[tracing::instrument(skip_all)]
    fn get_new_address(
        &self,
        _label: Option<&str>,
        _address_type: Option<json::AddressType>,
    ) -> bitcoincore_rpc::Result<Address<bitcoin::address::NetworkUnchecked>> {
        if _label.is_some() {
            return Err(Error::ReturnedError(format!(
                "This argument is unimplemented: {}",
                stringify!(_label)
            )));
        }
        if _address_type.is_some() {
            return Err(Error::ReturnedError(format!(
                "This argument is unimplemented: {}",
                stringify!(_address_type)
            )));
        }

        let address = ledger::Ledger::get_constant_credential_from_witness().address;

        Ok(address.as_unchecked().to_owned())
    }

    /// Generates `block_num` amount of block rewards to `address`. Also mines
    /// current mempool transactions to a block.
    #[tracing::instrument(skip_all)]
    fn generate_to_address(
        &self,
        block_num: u64,
        address: &Address<NetworkChecked>,
    ) -> bitcoincore_rpc::Result<Vec<bitcoin::BlockHash>> {
        let mut hashes: Vec<BlockHash> = Vec::new();

        for _ in 0..block_num {
            hashes.push(self.ledger.mine_block(address)?);
        }

        Ok(hashes)
    }

    /// This function is intended for retrieving information about a txout's
    /// value and confirmation. Other data may not be reliable.
    ///
    /// This will include mempool txouts regardless of the `include_mempool`
    /// flag. `coinbase` will be set to false regardless if it is or not.
    #[tracing::instrument(skip_all)]
    fn get_tx_out(
        &self,
        txid: &bitcoin::Txid,
        vout: u32,
        _include_mempool: Option<bool>,
    ) -> bitcoincore_rpc::Result<Option<json::GetTxOutResult>> {
        if let Some(false) = _include_mempool {
            return Err(Error::ReturnedError(format!(
                "This argument is unimplemented: {}",
                stringify!(_include_mempool)
            )));
        }

        let utxo = OutPoint { txid: *txid, vout };
        if self.ledger.is_utxo_spent(utxo) {
            return Err(LedgerError::Utxo(format!("UTXO {utxo:?} is spent")).into());
        }

        let bestblock = self.get_best_block_hash()?;

        let tx = self.get_raw_transaction(txid, None)?;
        let value = tx.output.get(vout as usize).unwrap().value;

        let confirmations = self.get_transaction(txid, None)?.info.confirmations as u32;

        Ok(Some(GetTxOutResult {
            bestblock,
            confirmations,
            value,
            script_pub_key: GetRawTransactionResultVoutScriptPubKey {
                asm: "TODO".to_string(),
                hex: Vec::new(),
                req_sigs: None,
                type_: None,
                addresses: Vec::new(),
                address: None,
            },
            coinbase: false,
        }))
    }

    #[tracing::instrument(skip_all)]
    fn get_best_block_hash(&self) -> bitcoincore_rpc::Result<bitcoin::BlockHash> {
        let current_height = self.ledger.get_block_height()?;
        let current_block = self.ledger.get_block_with_height(current_height)?;
        let block_hash = current_block.block_hash();

        Ok(block_hash)
    }

    #[tracing::instrument(skip_all)]
    fn get_block(&self, hash: &bitcoin::BlockHash) -> bitcoincore_rpc::Result<bitcoin::Block> {
        Ok(self.ledger.get_block_with_hash(*hash)?)
    }

    #[tracing::instrument(skip_all)]
    fn get_block_header(
        &self,
        hash: &bitcoin::BlockHash,
    ) -> bitcoincore_rpc::Result<bitcoin::block::Header> {
        Ok(self.ledger.get_block_with_hash(*hash)?.header)
    }

    #[tracing::instrument(skip_all)]
    fn get_block_count(&self) -> bitcoincore_rpc::Result<u64> {
        Ok(self.ledger.get_block_height()?.into())
    }

    #[tracing::instrument(skip_all)]
    fn fund_raw_transaction<R: bitcoincore_rpc::RawTx>(
        &self,
        tx: R,
        options: Option<&json::FundRawTransactionOptions>,
        _is_witness: Option<bool>,
    ) -> bitcoincore_rpc::Result<json::FundRawTransactionResult> {
        if _is_witness.is_some() {
            return Err(Error::ReturnedError(format!(
                "This argument is unimplemented: {}",
                stringify!(_is_witness)
            )));
        }

        let mut transaction: Transaction = encode::deserialize_hex(&tx.raw_hex())?;
        tracing::debug!("Decoded input transaction: {transaction:?}");

        let mut hex: Vec<u8> = Vec::new();
        let tx = encode_to_hex(&transaction);
        tx.consensus_encode(&mut hex).unwrap();

        let diff = match self.ledger.check_transaction_funds(&transaction) {
            // If input amount is sufficient, no need to modify anything.
            Ok(()) => {
                return Ok(json::FundRawTransactionResult {
                    hex,
                    fee: Amount::from_sat(0),
                    change_position: -1,
                })
            }
            // Input funds are lower than the output funds, use the difference.
            Err(LedgerError::InputFundsNotEnough(diff)) => diff,
            // Other ledger errors.
            Err(e) => return Err(e.into()),
        };

        tracing::debug!(
            "Input funds are {diff} sats lower than the output sats, adding new input."
        );

        // Generate a new txout.
        let address = self.get_new_address(None, None)?.assume_checked();
        let txid = self.send_to_address(
            &address,
            Amount::from_sat(diff * diff),
            None,
            None,
            None,
            None,
            None,
            None,
        )?;

        let txin = TxIn {
            previous_output: OutPoint { txid, vout: 0 },
            ..Default::default()
        };

        let insert_idx = match options {
            Some(option) => option
                .change_position
                .unwrap_or((transaction.input.len()) as u32),
            None => (transaction.input.len()) as u32,
        };

        transaction.input.insert(insert_idx as usize, txin);
        tracing::debug!("New transaction: {transaction:?}");

        let hex = serialize(&transaction);

        Ok(json::FundRawTransactionResult {
            hex,
            fee: Amount::from_sat(0),
            change_position: insert_idx as i32,
        })
    }

    #[tracing::instrument(skip_all)]
    fn sign_raw_transaction_with_wallet<R: bitcoincore_rpc::RawTx>(
        &self,
        tx: R,
        _utxos: Option<&[json::SignRawTransactionInput]>,
        _sighash_type: Option<json::SigHashType>,
    ) -> bitcoincore_rpc::Result<json::SignRawTransactionResult> {
        if _utxos.is_some() {
            return Err(Error::ReturnedError(format!(
                "This argument is unimplemented: {}",
                stringify!(_utxos)
            )));
        }
        if _sighash_type.is_some() {
            return Err(Error::ReturnedError(format!(
                "This argument is unimplemented: {}",
                stringify!(_sighash_type)
            )));
        }

        let mut transaction: Transaction = encode::deserialize_hex(&tx.raw_hex())?;
        tracing::debug!("Decoded input transaction: {transaction:?}");

        let credentials = ledger::Ledger::get_constant_credential_from_witness();

        let mut txouts: Vec<TxOut> = Vec::new();
        for input in transaction.input.clone() {
            let tx = match self.get_raw_transaction(&input.previous_output.txid, None) {
                Ok(tx) => tx,
                Err(e) => return Err(e),
            };

            let txout = match tx.output.get(input.previous_output.vout as usize) {
                Some(txout) => txout,
                None => {
                    return Err(LedgerError::Transaction(format!(
                        "No txout for {:?}",
                        input.previous_output
                    ))
                    .into())
                }
            };

            txouts.push(txout.clone());
        }

        let inputs: Vec<TxIn> = transaction
            .input
            .iter()
            .enumerate()
            .map(|(idx, input)| {
                let mut input = input.to_owned();
                tracing::trace!("Examining input {input:?}");

                if input.witness.is_empty()
                    && txouts[idx].script_pubkey == credentials.address.script_pubkey()
                {
                    tracing::debug!(
                        "Signing input {input:?} with witness {:?}",
                        credentials.witness.clone().unwrap()
                    );
                    input.witness = credentials.witness.clone().unwrap();
                }

                input
            })
            .collect();

        transaction.input = inputs;
        tracing::trace!("Final inputs {:?}", transaction.input);

        let hex = serialize(&transaction);

        Ok(SignRawTransactionResult {
            hex,
            complete: true,
            errors: None,
        })
    }

    #[tracing::instrument(skip_all)]
    fn get_chain_tips(&self) -> bitcoincore_rpc::Result<json::GetChainTipsResult> {
        let height = self.ledger.get_block_height().unwrap();
        let hash = if height == 0 {
            BlockHash::all_zeros()
        } else {
            self.ledger.get_block_with_height(height)?.block_hash()
        };

        let tip = json::GetChainTipsResultTip {
            height: height as u64,
            hash,
            branch_length: height as usize,
            status: GetChainTipsResultStatus::Active,
        };

        Ok(vec![tip])
    }

    #[tracing::instrument(skip_all)]
    fn get_block_hash(&self, height: u64) -> bitcoincore_rpc::Result<bitcoin::BlockHash> {
        Ok(self
            .ledger
            .get_block_with_height(height as u32)?
            .block_hash())
    }
}

#[cfg(test)]
mod tests {
    use crate::{ledger::Ledger, utils::_decode_from_hex, Client, RpcApiWrapper};
    use bitcoin::{
        consensus::{deserialize, Decodable},
        Amount, Network, OutPoint, Transaction, TxIn,
    };
    use bitcoincore_rpc::RpcApi;

    #[test]
    fn send_get_raw_transaction() {
        let rpc = Client::new("send_get_raw_transaction", bitcoincore_rpc::Auth::None).unwrap();

        let credential = Ledger::generate_credential_from_witness();
        let address = credential.address;

        // First, create a transaction for the next txin.
        let txout = rpc
            .ledger
            .create_txout(Amount::from_sat(100_000_000), address.script_pubkey());
        let tx = rpc.ledger.create_transaction(vec![], vec![txout]);
        let txid = rpc.ledger.add_transaction_unconditionally(tx).unwrap();

        // Create a new raw transactions that is valid.
        let txin = TxIn {
            previous_output: OutPoint { txid, vout: 0 },
            witness: credential.witness.clone().unwrap(),
            ..Default::default()
        };
        let txout = rpc
            .ledger
            .create_txout(Amount::from_sat(0x45), address.script_pubkey());
        let inserted_tx1 = rpc.ledger.create_transaction(vec![txin], vec![txout]);
        rpc.send_raw_transaction(&inserted_tx1).unwrap();

        let txin = TxIn {
            previous_output: OutPoint {
                txid: inserted_tx1.compute_txid(),
                vout: 0,
            },
            witness: credential.witness.unwrap(),
            ..Default::default()
        };
        let txout = rpc.ledger.create_txout(
            Amount::from_sat(0x45),
            Ledger::generate_credential_from_witness()
                .address
                .script_pubkey(),
        );
        let inserted_tx2 = rpc.ledger.create_transaction(vec![txin], vec![txout]);
        rpc.send_raw_transaction(&inserted_tx2).unwrap();

        // Retrieve inserted transactions from Bitcoin.
        let read_tx = rpc
            .get_raw_transaction(&inserted_tx1.compute_txid(), None)
            .unwrap();
        assert_eq!(read_tx, inserted_tx1);
        assert_ne!(read_tx, inserted_tx2);

        let read_tx = rpc
            .get_raw_transaction(&inserted_tx2.compute_txid(), None)
            .unwrap();
        assert_eq!(read_tx, inserted_tx2);
        assert_ne!(read_tx, inserted_tx1);
    }

    #[test]
    fn get_raw_transaction_info() {
        let rpc = Client::new("get_raw_transaction_info", bitcoincore_rpc::Auth::None).unwrap();

        let credential = Ledger::generate_credential_from_witness();
        let address = credential.address;

        let txout = rpc
            .ledger
            .create_txout(Amount::from_sat(0x45), address.script_pubkey());
        let tx = rpc.ledger.create_transaction(vec![], vec![txout]);
        let txid = rpc.ledger.add_transaction_unconditionally(tx).unwrap();

        // No blocks are mined. Little to none information is available.
        let info = rpc.get_raw_transaction_info(&txid, None).unwrap();
        assert_eq!(info.txid, txid);
        assert_eq!(info.blockhash, None);
        assert_eq!(info.confirmations, None);

        // Mining blocks should enable more transaction information.
        rpc.ledger.mine_block(&address).unwrap();
        let info = rpc.get_raw_transaction_info(&txid, None).unwrap();
        assert_eq!(info.txid, txid);
        assert_eq!(
            info.blockhash,
            Some(rpc.ledger.get_transaction_block_hash(&txid).unwrap())
        );
        assert_eq!(info.confirmations, Some(1));

        let txout = rpc
            .ledger
            .create_txout(Amount::from_sat(0x1F), address.script_pubkey());
        let tx = rpc.ledger.create_transaction(vec![], vec![txout]);
        let txid = rpc.ledger.add_transaction_unconditionally(tx).unwrap();

        // No blocks are mined. Little to none information is available.
        let info = rpc.get_raw_transaction_info(&txid, None).unwrap();
        assert_eq!(info.txid, txid);
        assert_eq!(info.blockhash, None);
        assert_eq!(info.confirmations, None);

        // Mining blocks should enable more transaction information.
        rpc.ledger.mine_block(&address).unwrap();
        rpc.ledger.mine_block(&address).unwrap();
        rpc.ledger.mine_block(&address).unwrap();
        let info = rpc.get_raw_transaction_info(&txid, None).unwrap();
        assert_eq!(info.txid, txid);
        assert_eq!(
            info.blockhash,
            Some(rpc.ledger.get_transaction_block_hash(&txid).unwrap())
        );
        assert_eq!(info.confirmations, Some(3));
    }

    #[test]
    fn get_transaction() {
        let rpc = Client::new("get_transaction", bitcoincore_rpc::Auth::None).unwrap();

        let credential = Ledger::generate_credential_from_witness();
        let address = credential.address;

        // First, add some funds to user, for free.
        let txout = rpc
            .ledger
            .create_txout(Amount::from_sat(100_000_000), address.script_pubkey());
        let tx = rpc.ledger.create_transaction(vec![], vec![txout]);
        let txid = rpc.ledger.add_transaction_unconditionally(tx).unwrap();

        // Insert raw transactions to Bitcoin.
        let txin = TxIn {
            previous_output: OutPoint { txid, vout: 0 },
            witness: credential.witness.unwrap(),
            ..Default::default()
        };
        let txout = rpc
            .ledger
            .create_txout(Amount::from_sat(0x1F), address.script_pubkey());
        let tx = rpc.ledger.create_transaction(vec![txin], vec![txout]);
        rpc.send_raw_transaction(&tx).unwrap();

        let txid = tx.compute_txid();

        let tx = rpc.get_transaction(&txid, None).unwrap();

        assert_eq!(txid, tx.info.txid);
    }

    #[test]
    fn send_to_address() {
        let rpc = Client::new("send_to_address", bitcoincore_rpc::Auth::None).unwrap();

        let credential = Ledger::generate_credential_from_witness();
        let receiver_address = credential.address;

        // send_to_address should send `amount` to `address`, regardless of the
        // user's balance.
        let txid = rpc
            .send_to_address(
                &receiver_address,
                Amount::from_sat(0x45),
                None,
                None,
                None,
                None,
                None,
                None,
            )
            .unwrap();

        let tx = rpc.get_raw_transaction(&txid, None).unwrap();

        // Receiver should have this.
        assert_eq!(tx.output[0].value.to_sat(), 0x45);
        assert_eq!(tx.output[0].script_pubkey, receiver_address.script_pubkey());
    }

    #[test]
    fn get_new_address() {
        let rpc = Client::new("get_new_address", bitcoincore_rpc::Auth::None).unwrap();

        let address = rpc.get_new_address(None, None).unwrap();

        assert!(address.is_valid_for_network(Network::Regtest));
        assert!(!address.is_valid_for_network(Network::Testnet));
        assert!(!address.is_valid_for_network(Network::Signet));
        assert!(!address.is_valid_for_network(Network::Bitcoin));
    }

    #[test]
    fn generate_to_address() {
        let rpc = Client::new("generate_to_address", bitcoincore_rpc::Auth::None).unwrap();

        let credential = Ledger::generate_credential_from_witness();
        let address = credential.address;

        // Empty wallet should reject transaction.
        let txout = rpc
            .ledger
            .create_txout(Amount::from_sat(1), address.script_pubkey());
        let tx = rpc.ledger.create_transaction(vec![], vec![txout]);
        assert!(rpc.ledger.check_transaction(&tx).is_err());

        // Generating blocks should add funds to wallet.
        rpc.generate_to_address(101, &address).unwrap();

        // Wallet has funds now. It should not be rejected.
        let txin = TxIn {
            previous_output: OutPoint {
                txid: rpc
                    .ledger
                    ._get_transactions()
                    .first()
                    .unwrap()
                    .compute_txid(),
                vout: 0,
            },
            witness: credential.witness.unwrap(),
            ..Default::default()
        };
        let txout = rpc
            .ledger
            .create_txout(Amount::from_sat(1), address.script_pubkey());
        let tx = rpc.ledger.create_transaction(vec![txin], vec![txout]);
        rpc.ledger.check_transaction(&tx).unwrap();
    }

    #[test]
    fn get_best_block_hash() {
        let rpc = Client::new("get_best_block_hash", bitcoincore_rpc::Auth::None).unwrap();
        let address = Ledger::generate_credential_from_witness().address;

        let tx = rpc.ledger.create_transaction(vec![], vec![]);
        rpc.ledger.add_transaction_unconditionally(tx).unwrap();
        let block_hash = rpc.ledger.mine_block(&address).unwrap();

        let best_block_hash = rpc.get_best_block_hash().unwrap();

        assert_eq!(block_hash, best_block_hash);
    }

    #[test]
    fn get_block() {
        let rpc = Client::new("get_block", bitcoincore_rpc::Auth::None).unwrap();
        let address = Ledger::generate_credential_from_witness().address;

        let tx = rpc.ledger.create_transaction(vec![], vec![]);
        rpc.ledger.add_transaction_unconditionally(tx).unwrap();
        let block_hash = rpc.ledger.mine_block(&address).unwrap();
        let block = rpc.ledger.get_block_with_hash(block_hash).unwrap();

        let read_block = rpc.get_block(&block_hash).unwrap();

        assert_eq!(block, read_block);
    }

    #[test]
    fn get_block_header() {
        let rpc = Client::new("get_block_header", bitcoincore_rpc::Auth::None).unwrap();
        let address = Ledger::generate_credential_from_witness().address;

        let tx = rpc.ledger.create_transaction(vec![], vec![]);
        rpc.ledger.add_transaction_unconditionally(tx).unwrap();
        let block_hash = rpc.ledger.mine_block(&address).unwrap();
        let block = rpc.ledger.get_block_with_hash(block_hash).unwrap();

        let block_header = rpc.get_block_header(&block_hash).unwrap();

        assert_eq!(block.header, block_header);
    }

    #[test]
    fn get_block_count() {
        let rpc = Client::new("get_block_count", bitcoincore_rpc::Auth::None).unwrap();
        let address = Ledger::generate_credential_from_witness().address;

        assert_eq!(rpc.get_block_count().unwrap(), 0);

        let tx = rpc.ledger.create_transaction(vec![], vec![]);
        rpc.ledger.add_transaction_unconditionally(tx).unwrap();
        rpc.ledger.mine_block(&address).unwrap();

        assert_eq!(rpc.get_block_count().unwrap(), 1);
    }

    #[test]
    fn fund_raw_transaction() {
        let rpc = Client::new("fund_raw_transaction", bitcoincore_rpc::Auth::None).unwrap();

        let address = Ledger::generate_credential_from_witness().address;
        let txid = rpc
            .send_to_address(
                &address,
                Amount::from_sat(0x1F),
                None,
                None,
                None,
                None,
                None,
                None,
            )
            .unwrap();
        let txin = rpc.ledger.create_txin(txid, 0);
        let txout = rpc
            .ledger
            .create_txout(Amount::from_sat(0x45), address.script_pubkey());
        let og_tx = rpc.ledger.create_transaction(vec![txin], vec![txout]);

        let res = rpc.fund_raw_transaction(&og_tx, None, None).unwrap();
        let tx = deserialize::<Transaction>(&res.hex).unwrap();

        assert_ne!(og_tx, tx);
        assert_ne!(res.change_position, -1);

        let res = rpc.fund_raw_transaction(&tx, None, None).unwrap();
        let new_tx = String::consensus_decode(&mut res.hex.as_slice()).unwrap();
        let new_tx = _decode_from_hex::<Transaction>(new_tx).unwrap();

        assert_eq!(tx, new_tx);
        assert_eq!(res.change_position, -1);
    }

    #[test]
    fn sign_raw_transaction_with_wallet() {
        let rpc = Client::new(
            "sign_raw_transaction_with_wallet",
            bitcoincore_rpc::Auth::None,
        )
        .unwrap();

        let address = Ledger::get_constant_credential_from_witness().address;
        let txid = rpc
            .send_to_address(
                &address,
                Amount::from_sat(0x1F),
                None,
                None,
                None,
                None,
                None,
                None,
            )
            .unwrap();
        let txin = TxIn {
            previous_output: OutPoint { txid, vout: 0 },
            script_sig: address.script_pubkey(),
            ..Default::default()
        };
        let txout = rpc
            .ledger
            .create_txout(Amount::from_sat(0x45), address.script_pubkey());
        let tx = rpc
            .ledger
            .create_transaction(vec![txin.clone()], vec![txout]);

        assert!(txin.witness.is_empty());

        let res = rpc
            .sign_raw_transaction_with_wallet(&tx, None, None)
            .unwrap();
        let new_tx = deserialize::<Transaction>(&res.hex).unwrap();

        assert!(!new_tx.input.first().unwrap().witness.is_empty());
    }
}
