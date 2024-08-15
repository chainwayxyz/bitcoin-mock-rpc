//! # RPC API
//!
//! This crate implements `RpcApi` trait in `bitcoincore-rpc` for the mock
//! `Client`.

use super::Client;
use crate::ledger::Ledger;
use bitcoin::{
    address::NetworkChecked,
    consensus::{encode, Encodable},
    hashes::Hash,
    params::Params,
    Address, Amount, BlockHash, SignedAmount, Transaction, Txid,
};
use bitcoincore_rpc::{
    json::{
        self, GetRawTransactionResult, GetRawTransactionResultVin,
        GetRawTransactionResultVinScriptSig, GetRawTransactionResultVout,
        GetRawTransactionResultVoutScriptPubKey, GetTransactionResult, GetTransactionResultDetail,
        GetTransactionResultDetailCategory, GetTxOutResult, WalletTxInfo,
    },
    RpcApi,
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
    fn call<T: for<'a> serde::de::Deserialize<'a>>(
        &self,
        cmd: &str,
        args: &[serde_json::Value],
    ) -> bitcoincore_rpc::Result<T> {
        unimplemented!(
            "Unimplemented mock RPC cmd: {}, with args: {:?}. Please consider implementing it.",
            cmd,
            args
        );
    }

    fn send_raw_transaction<R: bitcoincore_rpc::RawTx>(
        &self,
        tx: R,
    ) -> bitcoincore_rpc::Result<bitcoin::Txid> {
        let tx: Transaction = encode::deserialize_hex(&tx.raw_hex())?;

        self.ledger.add_transaction(tx.clone())?;

        Ok(tx.compute_txid())
    }
    fn get_raw_transaction(
        &self,
        txid: &bitcoin::Txid,
        _block_hash: Option<&bitcoin::BlockHash>,
    ) -> bitcoincore_rpc::Result<bitcoin::Transaction> {
        Ok(self.ledger.get_transaction(*txid)?)
    }
    /// Verbose flag enabled `get_raw_transaction`.
    ///
    /// This function **is not** intended to return information about the
    /// transaction itself. Instead, it mostly provides information about the
    /// transaction's state in blockchain. It is recommmended to use
    /// `get_raw_transaction` for information about transaction's inputs and
    /// outputs.
    fn get_raw_transaction_info(
        &self,
        txid: &bitcoin::Txid,
        _block_hash: Option<&bitcoin::BlockHash>,
    ) -> bitcoincore_rpc::Result<json::GetRawTransactionResult> {
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

    fn get_transaction(
        &self,
        txid: &bitcoin::Txid,
        _include_watchonly: Option<bool>,
    ) -> bitcoincore_rpc::Result<json::GetTransactionResult> {
        let raw_tx = self.get_raw_transaction(txid, None).unwrap();
        let mut amount = Amount::from_sat(0);

        let details: Vec<GetTransactionResultDetail> = raw_tx
            .output
            .iter()
            .map(|output| {
                amount += output.value;
                GetTransactionResultDetail {
                    address: Some(
                        Address::from_script(
                            &output.script_pubkey,
                            Params::new(bitcoin::Network::Regtest),
                        )
                        .unwrap()
                        .as_unchecked()
                        .clone(),
                    ),
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
    fn get_new_address(
        &self,
        _label: Option<&str>,
        _address_type: Option<json::AddressType>,
    ) -> bitcoincore_rpc::Result<Address<bitcoin::address::NetworkUnchecked>> {
        let address = Ledger::generate_address_from_witness();

        Ok(address.as_unchecked().to_owned())
    }

    /// Generates `block_num` amount of block rewards to `address`. Block reward
    /// is fixed to 1 BTC, regardless of which and how many blocks are
    /// generated. Also mines current mempool transactions to a block.
    fn generate_to_address(
        &self,
        block_num: u64,
        address: &Address<NetworkChecked>,
    ) -> bitcoincore_rpc::Result<Vec<bitcoin::BlockHash>> {
        let mut hashes: Vec<BlockHash> = Vec::new();

        for _ in 0..block_num {
            self.send_to_address(
                address,
                Amount::from_sat(100_000_000),
                None,
                None,
                None,
                None,
                None,
                None,
            )?;

            hashes.push(self.ledger.mine_block(address)?);
        }

        Ok(hashes)
    }

    /// This function is intended for retrieving information about a txout's
    /// value and confirmation. Other data may not be reliable.
    ///
    /// This will include mempool txouts regardless of the `include_mempool`
    /// flag. `coinbase` will be set to false regardless if it is or not.
    fn get_tx_out(
        &self,
        txid: &bitcoin::Txid,
        vout: u32,
        _include_mempool: Option<bool>,
    ) -> bitcoincore_rpc::Result<Option<json::GetTxOutResult>> {
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

    fn get_best_block_hash(&self) -> bitcoincore_rpc::Result<bitcoin::BlockHash> {
        let current_height = self.ledger.get_block_height()?;
        let current_block = self.ledger.get_block_with_height(current_height)?;
        let block_hash = current_block.block_hash();

        Ok(block_hash)
    }

    fn get_block(&self, hash: &bitcoin::BlockHash) -> bitcoincore_rpc::Result<bitcoin::Block> {
        Ok(self.ledger.get_block_with_hash(*hash)?)
    }

    fn get_block_header(
        &self,
        hash: &bitcoin::BlockHash,
    ) -> bitcoincore_rpc::Result<bitcoin::block::Header> {
        Ok(self.ledger.get_block_with_hash(*hash)?.header)
    }

    fn get_block_count(&self) -> bitcoincore_rpc::Result<u64> {
        Ok(self.ledger.get_block_height()?.into())
    }
}

#[cfg(test)]
mod tests {
    use crate::{ledger::Ledger, Client, RpcApiWrapper};
    use bitcoin::{Amount, Network, OutPoint, TxIn};
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
        if let Ok(()) = rpc.ledger.check_transaction(&tx) {
            assert!(false);
        };

        // Generating blocks should add funds to wallet.
        rpc.generate_to_address(101, &address).unwrap();

        // Wallet has funds now. It should not be rejected.
        let txin = TxIn {
            previous_output: OutPoint {
                txid: rpc.ledger.get_transactions().get(0).unwrap().compute_txid(),
                vout: 0,
            },
            witness: credential.witness.unwrap(),
            ..Default::default()
        };
        let txout = rpc
            .ledger
            .create_txout(Amount::from_sat(1), address.script_pubkey());
        let tx = rpc.ledger.create_transaction(vec![txin], vec![txout]);
        if let Err(e) = rpc.ledger.check_transaction(&tx) {
            assert!(false, "{:?}", e);
        };
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
}
