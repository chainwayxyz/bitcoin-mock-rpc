//! # RPC API
//!
//! This crate implements `RpcApi` trait in `bitcoincore-rpc` for the mock
//! `Client`.

use super::Client;
use bitcoin::{
    absolute, address::NetworkChecked, consensus::encode, hashes::Hash, Address, Amount, BlockHash,
    Network, SignedAmount, Transaction, TxIn, TxOut, Wtxid, XOnlyPublicKey,
};
use bitcoincore_rpc::{
    json::{
        self, GetRawTransactionResult, GetTransactionResult, GetTransactionResultDetail,
        GetTransactionResultDetailCategory, WalletTxInfo,
    },
    RpcApi,
};
use secp256k1::{rand, Keypair, Secp256k1};

impl RpcApi for Client {
    /// This function normally talks with Bitcoin network. Therefore, other
    /// functions calls this to send requests. In a mock environment though,
    /// other functions won't be talking to a regulated interface. Rather will
    /// talk with a temporary interfaces, like in-memory databases.
    ///
    /// This is the reason, this function will only throw errors in case of a
    /// function is not -yet- implemented. Tester should implement corresponding
    /// function in this impl block, if this function called for `cmd`.
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

    fn get_raw_transaction(
        &self,
        txid: &bitcoin::Txid,
        _block_hash: Option<&bitcoin::BlockHash>,
    ) -> bitcoincore_rpc::Result<bitcoin::Transaction> {
        Ok(self
            .database
            .lock()
            .unwrap()
            .get_transaction(&txid.to_string())
            .unwrap())
    }

    fn send_raw_transaction<R: bitcoincore_rpc::RawTx>(
        &self,
        tx: R,
    ) -> bitcoincore_rpc::Result<bitcoin::Txid> {
        let tx: Transaction = encode::deserialize_hex(&tx.raw_hex()).unwrap();

        self.database
            .lock()
            .unwrap()
            .insert_transaction_unconditionally(&tx)
            .unwrap();

        Ok(tx.compute_txid())
    }

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
        let txin = TxIn::default();
        let txout = TxOut {
            value: amount,
            script_pubkey: address.script_pubkey(),
        };
        let tx = bitcoin::Transaction {
            version: bitcoin::transaction::Version(2),
            lock_time: absolute::LockTime::from_consensus(0),
            input: vec![txin],
            output: vec![txout],
        };

        let txid = self.send_raw_transaction(&tx)?;

        Ok(txid)
    }

    fn get_transaction(
        &self,
        txid: &bitcoin::Txid,
        _include_watchonly: Option<bool>,
    ) -> bitcoincore_rpc::Result<json::GetTransactionResult> {
        let raw_tx = self.get_raw_transaction(txid, None).unwrap();

        let res = GetTransactionResult {
            info: WalletTxInfo {
                confirmations: i32::MAX,
                blockhash: None,
                blockindex: None,
                blocktime: Some(0),
                blockheight: None,
                txid: *txid,
                time: 0,
                timereceived: 0,
                bip125_replaceable: json::Bip125Replaceable::Unknown,
                wallet_conflicts: vec![],
            },
            amount: SignedAmount::from_sat(raw_tx.output[0].value.to_sat() as i64),
            fee: None,
            details: vec![GetTransactionResultDetail {
                address: None,
                category: GetTransactionResultDetailCategory::Send,
                amount: SignedAmount::from_sat(raw_tx.output[0].value.to_sat() as i64),
                label: None,
                vout: 0,
                fee: None,
                abandoned: None,
            }],
            hex: encode::serialize(&raw_tx),
        };

        Ok(res)
    }

    fn get_new_address(
        &self,
        _label: Option<&str>,
        _address_type: Option<json::AddressType>,
    ) -> bitcoincore_rpc::Result<Address<bitcoin::address::NetworkUnchecked>> {
        let secp = Secp256k1::new();
        let (sk, _pk) = secp.generate_keypair(&mut rand::thread_rng());
        let (xonly_public_key, _parity) =
            XOnlyPublicKey::from_keypair(&Keypair::from_secret_key(&secp, &sk));

        let address = Address::p2tr(&secp, xonly_public_key, None, Network::Regtest)
            .as_unchecked()
            .to_owned();

        self.ledger.set(
            self.ledger
                .take()
                .add_address(address.clone().assume_checked()),
        );

        Ok(address)
    }

    fn get_raw_transaction_info(
        &self,
        txid: &bitcoin::Txid,
        _block_hash: Option<&bitcoin::BlockHash>,
    ) -> bitcoincore_rpc::Result<json::GetRawTransactionResult> {
        Ok(GetRawTransactionResult {
            in_active_chain: None,
            hex: vec![],
            txid: *txid,
            hash: Wtxid::hash(&[0]),
            size: 0,
            vsize: 0,
            version: 0,
            locktime: 0,
            vin: vec![],
            vout: vec![],
            blockhash: None,
            confirmations: Some(10),
            time: None,
            blocktime: None,
        })
    }

    /// Generates `block_num` amount of block rewards to user.
    fn generate_to_address(
        &self,
        block_num: u64,
        address: &Address<NetworkChecked>,
    ) -> bitcoincore_rpc::Result<Vec<bitcoin::BlockHash>> {
        // Block reward is 1 BTC regardless of how many block is mined.
        let txout = TxOut {
            value: Amount::from_sat(100_000_000 * block_num),
            script_pubkey: address.script_pubkey(),
        };
        let tx = Transaction {
            version: bitcoin::transaction::Version(2),
            lock_time: absolute::LockTime::from_consensus(0),
            input: vec![],
            output: vec![txout],
        };

        self.database
            .lock()
            .unwrap()
            .insert_transaction_unconditionally(&tx)
            .unwrap();

        for output in tx.output {
            self.ledger.set(
                self.ledger
                    .take()
                    .add_utxo(output),
            );
        }

        Ok(vec![BlockHash::all_zeros(); block_num as usize])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_common::*;
    use bitcoin::{hashes::Hash, Amount, OutPoint, ScriptBuf, TxIn, TxOut, Txid, Witness};

    /// Tests `send_raw_transaction` and `get_raw_transaction`.
    #[test]
    fn raw_transaction() {
        let rpc = Client::new("", bitcoincore_rpc::Auth::None).unwrap();

        // Insert raw transactions to Bitcoin.
        let txin = TxIn {
            previous_output: OutPoint {
                txid: Txid::from_byte_array([0x45; 32]),
                vout: 0,
            },
            sequence: bitcoin::transaction::Sequence::ENABLE_RBF_NO_LOCKTIME,
            script_sig: ScriptBuf::default(),
            witness: Witness::new(),
        };
        let txout = TxOut {
            value: Amount::from_sat(0x1F),
            script_pubkey: get_temp_address().script_pubkey(),
        };
        let inserted_tx1 = create_transaction(vec![txin], vec![txout]);
        rpc.send_raw_transaction(&inserted_tx1).unwrap();

        let txin = TxIn {
            previous_output: OutPoint {
                txid: inserted_tx1.compute_txid(),
                vout: 0,
            },
            sequence: bitcoin::transaction::Sequence::ENABLE_RBF_NO_LOCKTIME,
            script_sig: ScriptBuf::default(),
            witness: Witness::new(),
        };
        let txout = TxOut {
            value: Amount::from_sat(0x45),
            script_pubkey: get_temp_address().script_pubkey(),
        };
        let inserted_tx2 = create_transaction(vec![txin], vec![txout]);
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

    /// Tests `get_transaction`.
    #[test]
    fn transaction() {
        let rpc = Client::new("", bitcoincore_rpc::Auth::None).unwrap();

        // Insert raw transactions to Bitcoin.
        let txin = TxIn {
            previous_output: OutPoint {
                txid: Txid::from_byte_array([0x45; 32]),
                vout: 0,
            },
            sequence: bitcoin::transaction::Sequence::ENABLE_RBF_NO_LOCKTIME,
            script_sig: ScriptBuf::default(),
            witness: Witness::new(),
        };
        let txout = TxOut {
            value: Amount::from_sat(0x1F),
            script_pubkey: get_temp_address().script_pubkey(),
        };
        let inserted_tx = create_transaction(vec![txin], vec![txout]);
        rpc.send_raw_transaction(&inserted_tx).unwrap();

        let txid = inserted_tx.compute_txid();

        let tx = rpc.get_transaction(&txid, None).unwrap();

        assert_eq!(txid, tx.info.txid);
    }

    #[test]
    fn send_to_address() {
        let rpc = Client::new("", bitcoincore_rpc::Auth::None).unwrap();

        let address = get_temp_address();

        let txid = rpc
            .send_to_address(
                &address,
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
        assert_eq!(tx.output[0].value.to_sat(), 0x45);
    }

    #[test]
    fn get_new_address() {
        let rpc = Client::new("", bitcoincore_rpc::Auth::None).unwrap();

        let address = rpc.get_new_address(None, None).unwrap();

        assert!(address.is_valid_for_network(Network::Regtest));
        assert!(!address.is_valid_for_network(Network::Testnet));
        assert!(!address.is_valid_for_network(Network::Signet));
        assert!(!address.is_valid_for_network(Network::Bitcoin));
        unsafe { assert_eq!(*(*rpc.ledger.as_ptr()).addresses[0].as_unchecked(), address) };

        const ADDRESS_COUNT: usize = 100;
        let mut prev = address;
        for i in 0..ADDRESS_COUNT {
            let curr = rpc.get_new_address(None, None).unwrap();

            assert_ne!(prev, curr);
            assert!(curr.is_valid_for_network(Network::Regtest));
            assert!(!curr.is_valid_for_network(Network::Testnet));
            assert!(!curr.is_valid_for_network(Network::Signet));
            assert!(!curr.is_valid_for_network(Network::Bitcoin));
            unsafe {
                assert_eq!(
                    *(*rpc.ledger.as_ptr()).addresses[i + 1].as_unchecked(),
                    curr
                )
            };

            prev = curr;
        }
    }

    #[test]
    fn generate_to_address() {
        let rpc = Client::new("", bitcoincore_rpc::Auth::None).unwrap();

        let address = rpc.get_new_address(None, None).unwrap().assume_checked();

        // Empty wallet should reject transaction.
        let txout = TxOut {
            value: Amount::from_sat(1),
            script_pubkey: address.script_pubkey(),
        };
        let tx = Transaction {
            version: bitcoin::transaction::Version(2),
            lock_time: absolute::LockTime::from_consensus(0),
            input: vec![],
            output: vec![txout],
        };
        if let Ok(()) = rpc.database.lock().unwrap().verify_transaction(&tx) {
            assert!(false);
        };

        rpc.generate_to_address(101, &address).unwrap();

        if let Err(_) = rpc.database.lock().unwrap().verify_transaction(&tx) {
            assert!(false);
        };
    }
}
