//! # RPC API
//!
//! This crate implements `RpcApi` trait in `bitcoincore-rpc` for the mock
//! `Client`.

use super::Client;
use crate::ledger::Ledger;
use bitcoin::{
    address::NetworkChecked, consensus::encode, hashes::Hash, params::Params, Address, Amount,
    BlockHash, SignedAmount, Transaction, Wtxid,
};
use bitcoincore_rpc::{
    json::{
        self, GetRawTransactionResult, GetRawTransactionResultVoutScriptPubKey,
        GetTransactionResult, GetTransactionResultDetail, GetTransactionResultDetailCategory,
        GetTxOutResult, WalletTxInfo,
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
        let tx: Transaction = encode::deserialize_hex(&tx.raw_hex()).unwrap();

        self.ledger.add_transaction(tx.clone())?;

        Ok(tx.compute_txid())
    }
    /// Because there are no blocks, this function works pretty much same as
    /// `get_transaction`.
    fn get_raw_transaction(
        &self,
        txid: &bitcoin::Txid,
        _block_hash: Option<&bitcoin::BlockHash>,
    ) -> bitcoincore_rpc::Result<bitcoin::Transaction> {
        Ok(self.ledger.get_transaction(*txid)?)
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

        Ok(GetTransactionResult {
            info: WalletTxInfo {
                confirmations: i32::MAX - 1,
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
            amount: SignedAmount::from_sat(amount.to_sat() as i64),
            fee: None,
            details,
            hex: encode::serialize(&raw_tx),
        })
    }

    /// Warning `send_to_address` won't check anything. It will only send funds
    /// to specified address. This means: Unlimited free money.
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
        let txout = self.ledger.create_txout(
            Amount::from_sat(rand::thread_rng().next_u64()) + amount,
            address.script_pubkey(),
        );
        let tx = self.ledger.create_transaction(vec![], vec![txout]);
        let txid = tx.compute_txid();
        self.ledger.add_transaction_unconditionally(tx)?;

        // Now send amount to address.
        let txin = self.ledger.create_txin(txid, 0);
        let txout = self.ledger.create_txout(amount, address.script_pubkey());
        let tx = self.ledger.create_transaction(vec![txin], vec![txout]);

        Ok(self.ledger.add_transaction_unconditionally(tx)?)
    }

    // / Creates a random secret/public key pair and generates a Bitcoin address
    // / from witness program.
    fn get_new_address(
        &self,
        _label: Option<&str>,
        _address_type: Option<json::AddressType>,
    ) -> bitcoincore_rpc::Result<Address<bitcoin::address::NetworkUnchecked>> {
        let address = Ledger::generate_address_from_witness();

        Ok(address.as_unchecked().to_owned())
    }

    /// Generates `block_num` amount of block rewards to user. Block reward is
    /// fixed to 1 BTC, regardless of which and how many blocks are generated.
    fn generate_to_address(
        &self,
        block_num: u64,
        address: &Address<NetworkChecked>,
    ) -> bitcoincore_rpc::Result<Vec<bitcoin::BlockHash>> {
        // Block reward is 1 BTC regardless of how many block is mined.
        let txout = self.ledger.create_txout(
            Amount::from_sat(100_000_000 * block_num),
            address.script_pubkey(),
        );
        let tx = self.ledger.create_transaction(vec![], vec![txout]);

        self.ledger.add_transaction_unconditionally(tx)?;

        // Finally, increase the block height.
        let current_height = self.ledger.get_block_height();
        self.ledger.set_block_height(current_height + block_num);

        Ok(vec![BlockHash::all_zeros(); block_num as usize])
    }

    /// TODO: whole function
    fn get_tx_out(
        &self,
        _txid: &bitcoin::Txid,
        _vout: u32,
        _include_mempool: Option<bool>,
    ) -> bitcoincore_rpc::Result<Option<json::GetTxOutResult>> {
        Ok(Some(GetTxOutResult {
            bestblock: BlockHash::all_zeros(),
            confirmations: u32::MAX,
            value: Amount::from_sat(0x45),
            script_pub_key: GetRawTransactionResultVoutScriptPubKey {
                asm: "asmwhat".to_string(),
                hex: Vec::new(),
                req_sigs: None,
                type_: None,
                addresses: Vec::new(),
                address: None,
            },
            coinbase: true,
        }))
    }

    // / Returns user's balance. Balance is calculated using addresses that are
    // / generated with `get_new_address` rpc call.
    // fn get_balance(
    //     &self,
    //     _minconf: Option<usize>,
    //     _include_watchonly: Option<bool>,
    // ) -> bitcoincore_rpc::Result<Amount> {
    //     Ok(self.ledger.calculate_balance()?)
    // }
}

#[cfg(test)]
mod tests {
    #[test]
    fn raw_transaction() {
        // let rpc = Client::new("", bitcoincore_rpc::Auth::None).unwrap();

        // let credential = Ledger::generate_credential_from_witness();
        // rpc.ledger.add_credential(credential.clone());
        // let address = credential.address;

        // // First, add some funds to user, for free.
        // let txout = rpc
        //     .ledger
        //     .create_txout(Amount::from_sat(100_000_000), address.script_pubkey());
        // let tx = rpc.ledger.create_transaction(vec![], vec![txout]);
        // let txid = rpc.ledger.add_transaction_unconditionally(tx).unwrap();

        // // Create a new raw transactions that is valid.
        // let txin = rpc.ledger._create_txin(txid, 0);
        // let txout = rpc
        //     .ledger
        //     .create_txout(Amount::from_sat(0x45), address.script_pubkey());
        // let inserted_tx1 = rpc.ledger.create_transaction(vec![txin], vec![txout]);
        // rpc.send_raw_transaction(&inserted_tx1).unwrap();

        // let txin = rpc.ledger._create_txin(inserted_tx1.compute_txid(), 0);
        // let txout = rpc.ledger.create_txout(
        //     Amount::from_sat(0x45),
        //     Ledger::generate_credential_from_witness()
        //         .address
        //         .script_pubkey(),
        // );
        // let inserted_tx2 = rpc.ledger.create_transaction(vec![txin], vec![txout]);
        // rpc.send_raw_transaction(&inserted_tx2).unwrap();

        // // Retrieve inserted transactions from Bitcoin.
        // let read_tx = rpc
        //     .get_raw_transaction(&inserted_tx1.compute_txid(), None)
        //     .unwrap();
        // assert_eq!(read_tx, inserted_tx1);
        // assert_ne!(read_tx, inserted_tx2);

        // let read_tx = rpc
        //     .get_raw_transaction(&inserted_tx2.compute_txid(), None)
        //     .unwrap();
        // assert_eq!(read_tx, inserted_tx2);
        // assert_ne!(read_tx, inserted_tx1);
    }

    #[test]
    fn transaction() {
        // let rpc = Client::new("", bitcoincore_rpc::Auth::None).unwrap();

        // let credential = Ledger::generate_credential_from_witness();
        // rpc.ledger.add_credential(credential.clone());
        // let address = credential.address;

        // // First, add some funds to user, for free.
        // let txout = rpc
        //     .ledger
        //     .create_txout(Amount::from_sat(100_000_000), address.script_pubkey());
        // let tx = rpc.ledger.create_transaction(vec![], vec![txout]);
        // let txid = rpc.ledger.add_transaction_unconditionally(tx).unwrap();

        // // Insert raw transactions to Bitcoin.
        // let txin = rpc.ledger._create_txin(txid, 0);
        // let txout = rpc
        //     .ledger
        //     .create_txout(Amount::from_sat(0x1F), address.script_pubkey());
        // let tx = rpc.ledger.create_transaction(vec![txin], vec![txout]);
        // rpc.send_raw_transaction(&tx).unwrap();

        // let txid = tx.compute_txid();

        // let tx = rpc.get_transaction(&txid, None).unwrap();

        // assert_eq!(txid, tx.info.txid);
    }

    #[test]
    fn send_to_address() {
        // let rpc = Client::new("", bitcoincore_rpc::Auth::None).unwrap();

        // let credential = Ledger::generate_credential_from_witness();
        // let receiver_address = credential.address;

        // // send_to_address should send `amount` to `address`, regardless of the
        // // user's balance.
        // let txid = rpc
        //     .send_to_address(
        //         &receiver_address,
        //         Amount::from_sat(0x45),
        //         None,
        //         None,
        //         None,
        //         None,
        //         None,
        //         None,
        //     )
        //     .unwrap();

        // let tx = rpc.get_raw_transaction(&txid, None).unwrap();

        // // Receiver should have this.
        // assert_eq!(tx.output[0].value.to_sat(), 0x45);
        // assert_eq!(tx.output[0].script_pubkey, receiver_address.script_pubkey());
    }

    // #[test]
    // fn get_new_address() {
    //     let rpc = Client::new("", bitcoincore_rpc::Auth::None).unwrap();

    //     let address = rpc.get_new_address(None, None).unwrap();

    //     assert!(address.is_valid_for_network(Network::Regtest));
    //     assert!(!address.is_valid_for_network(Network::Testnet));
    //     assert!(!address.is_valid_for_network(Network::Signet));
    //     assert!(!address.is_valid_for_network(Network::Bitcoin));
    //     assert_eq!(
    //         *rpc.ledger.get_credentials()[0].address.as_unchecked(),
    //         address
    //     );

    //     const ADDRESS_COUNT: usize = 100;
    //     let mut prev = address;
    //     for i in 0..ADDRESS_COUNT {
    //         let curr = rpc.get_new_address(None, None).unwrap();

    //         assert_ne!(prev, curr);
    //         assert!(curr.is_valid_for_network(Network::Regtest));
    //         assert!(!curr.is_valid_for_network(Network::Testnet));
    //         assert!(!curr.is_valid_for_network(Network::Signet));
    //         assert!(!curr.is_valid_for_network(Network::Bitcoin));
    //         assert_eq!(
    //             *rpc.ledger.get_credentials()[i + 1].address.as_unchecked(),
    //             curr
    //         );

    //         prev = curr;
    //     }
    // }

    // #[test]
    // fn generate_to_address() {
    //     let rpc = Client::new("", bitcoincore_rpc::Auth::None).unwrap();

    //     let address = rpc.get_new_address(None, None).unwrap().assume_checked();

    //     // Empty wallet should reject transaction.
    //     let txout = rpc
    //         .ledger
    //         .create_txout(Amount::from_sat(1), address.script_pubkey());
    //     let tx = rpc.ledger.create_transaction(vec![], vec![txout]);
    //     if let Ok(()) = rpc.ledger.check_transaction(&tx) {
    //         assert!(false);
    //     };

    //     // Generating blocks should add funds to wallet.
    //     rpc.generate_to_address(101, &address).unwrap();

    //     // Wallet has funds now. It should not be rejected.
    //     let txin = rpc.ledger._create_txin(
    //         rpc.ledger.get_transactions().get(0).unwrap().compute_txid(),
    //         0,
    //     );
    //     let txout = rpc
    //         .ledger
    //         .create_txout(Amount::from_sat(1), address.script_pubkey());
    //     let tx = rpc.ledger.create_transaction(vec![txin], vec![txout]);
    //     if let Err(e) = rpc.ledger.check_transaction(&tx) {
    //         assert!(false, "{:?}", e);
    //     };
    // }

    // #[test]
    // fn get_balance() {
    //     let rpc = Client::new("", bitcoincore_rpc::Auth::None).unwrap();

    //     let credential = Ledger::generate_credential_from_witness();
    //     rpc.ledger.add_credential(credential.clone());
    //     let address = credential.address;

    //     assert_eq!(rpc.get_balance(None, None).unwrap(), Amount::from_sat(0));

    //     let txout = rpc
    //         .ledger
    //         .create_txout(Amount::from_sat(0x45), address.script_pubkey());
    //     let tx = rpc.ledger.create_transaction(vec![], vec![txout]);
    //     rpc.ledger.add_transaction_unconditionally(tx).unwrap();

    //     assert_eq!(rpc.get_balance(None, None).unwrap(), Amount::from_sat(0x45));
    // }
}
