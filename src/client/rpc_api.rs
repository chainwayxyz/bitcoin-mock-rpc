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
        self, GetRawTransactionResult, GetTransactionResult, GetTransactionResultDetail,
        GetTransactionResultDetailCategory, WalletTxInfo,
    },
    RpcApi,
};

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

    fn send_raw_transaction<R: bitcoincore_rpc::RawTx>(
        &self,
        tx: R,
    ) -> bitcoincore_rpc::Result<bitcoin::Txid> {
        let tx: Transaction = encode::deserialize_hex(&tx.raw_hex()).unwrap();

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

        let details: Vec<GetTransactionResultDetail> = raw_tx
            .output
            .iter()
            .map(|utxo| GetTransactionResultDetail {
                address: Some(
                    Address::from_script(
                        &utxo.script_pubkey,
                        Params::new(bitcoin::Network::Regtest),
                    )
                    .unwrap()
                    .as_unchecked()
                    .clone(),
                ),
                category: GetTransactionResultDetailCategory::Send,
                amount: SignedAmount::from_sat(utxo.value.to_sat() as i64),
                label: None,
                vout: 0,
                fee: None,
                abandoned: None,
            })
            .collect();

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
            details,
            hex: encode::serialize(&raw_tx),
        };

        Ok(res)
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
        let target_txout = self
            .ledger
            .create_txout(amount, Some(address.script_pubkey()));

        let tx = self.ledger.create_transaction(vec![], vec![target_txout]);

        Ok(self.ledger.add_transaction_unconditionally(tx.clone())?)
    }

    fn get_new_address(
        &self,
        _label: Option<&str>,
        _address_type: Option<json::AddressType>,
    ) -> bitcoincore_rpc::Result<Address<bitcoin::address::NetworkUnchecked>> {
        let credential = Ledger::generate_credential_from_witness();
        self.ledger.add_credential(credential.clone());

        Ok(credential.address.as_unchecked().to_owned())
    }

    /// Generates `block_num` amount of block rewards to user.
    fn generate_to_address(
        &self,
        block_num: u64,
        address: &Address<NetworkChecked>,
    ) -> bitcoincore_rpc::Result<Vec<bitcoin::BlockHash>> {
        // Block reward is 1 BTC regardless of how many block is mined.
        let txout = self.ledger.create_txout(
            Amount::from_sat(100_000_000 * block_num),
            Some(address.script_pubkey()),
        );
        let tx = self.ledger.create_transaction(vec![], vec![txout]);

        self.ledger.add_transaction_unconditionally(tx.clone())?;

        Ok(vec![BlockHash::all_zeros(); block_num as usize])
    }

    fn get_balance(
        &self,
        _minconf: Option<usize>,
        _include_watchonly: Option<bool>,
    ) -> bitcoincore_rpc::Result<Amount> {
        Ok(self.ledger.calculate_balance()?)
    }

    fn list_unspent(
        &self,
        _minconf: Option<usize>,
        _maxconf: Option<usize>,
        _addresses: Option<&[&Address<NetworkChecked>]>,
        _include_unsafe: Option<bool>,
        _query_options: Option<json::ListUnspentQueryOptions>,
    ) -> bitcoincore_rpc::Result<Vec<json::ListUnspentResultEntry>> {
        let utxos = self.ledger.get_utxos();

        Ok(utxos
            .iter()
            .map(|utxo| {
                let tx = self.ledger.get_transaction(utxo.txid).unwrap();
                let output = tx.output.get(utxo.vout as usize).unwrap();

                json::ListUnspentResultEntry {
                    txid: utxo.txid,
                    vout: utxo.vout,
                    address: None,
                    label: None,
                    redeem_script: None,
                    witness_script: None,
                    script_pub_key: output.script_pubkey.clone(),
                    amount: output.value,
                    confirmations: 101,
                    spendable: true,
                    solvable: true,
                    descriptor: None,
                    safe: true,
                }
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RpcApiWrapper;
    use bitcoin::{Amount, Network};

    #[test]
    fn raw_transaction() {
        let rpc = Client::new("", bitcoincore_rpc::Auth::None).unwrap();

        let credential = Ledger::generate_credential_from_witness();
        rpc.ledger.add_credential(credential.clone());
        let address = credential.address;

        // First, add some funds to user, for free.
        let txout = rpc
            .ledger
            .create_txout(Amount::from_sat(100_000_000), Some(address.script_pubkey()));
        let tx = rpc.ledger.create_transaction(vec![], vec![txout]);
        let txid = rpc.ledger.add_transaction_unconditionally(tx).unwrap();

        // Create a new raw transactions that is valid.
        let txin = rpc.ledger._create_txin(txid, 0);
        let txout = rpc
            .ledger
            .create_txout(Amount::from_sat(0x45), Some(address.script_pubkey()));
        let inserted_tx1 = rpc.ledger.create_transaction(vec![txin], vec![txout]);
        rpc.send_raw_transaction(&inserted_tx1).unwrap();

        let txin = rpc.ledger._create_txin(inserted_tx1.compute_txid(), 0);
        let txout = rpc.ledger.create_txout(
            Amount::from_sat(0x45),
            Some(
                Ledger::generate_credential_from_witness()
                    .address
                    .script_pubkey(),
            ),
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
    fn transaction() {
        let rpc = Client::new("", bitcoincore_rpc::Auth::None).unwrap();

        let credential = Ledger::generate_credential_from_witness();
        rpc.ledger.add_credential(credential.clone());
        let address = credential.address;

        // First, add some funds to user, for free.
        let txout = rpc
            .ledger
            .create_txout(Amount::from_sat(100_000_000), Some(address.script_pubkey()));
        let tx = rpc.ledger.create_transaction(vec![], vec![txout]);
        let txid = rpc.ledger.add_transaction_unconditionally(tx).unwrap();

        // Insert raw transactions to Bitcoin.
        let txin = rpc.ledger._create_txin(txid, 0);
        let txout = rpc
            .ledger
            .create_txout(Amount::from_sat(0x1F), Some(address.script_pubkey()));
        let tx = rpc.ledger.create_transaction(vec![txin], vec![txout]);
        rpc.send_raw_transaction(&tx).unwrap();

        let txid = tx.compute_txid();

        let tx = rpc.get_transaction(&txid, None).unwrap();

        assert_eq!(txid, tx.info.txid);
    }

    #[test]
    fn send_to_address() {
        let rpc = Client::new("", bitcoincore_rpc::Auth::None).unwrap();

        let credential = Ledger::generate_credential_from_witness();
        rpc.ledger.add_credential(credential.clone());
        let address = credential.address;

        let credential = Ledger::generate_credential_from_witness();
        let receiver_address = credential.address;

        // Add small UTXO's to user.
        for i in 0..100 {
            let txout = rpc
                .ledger
                .create_txout(Amount::from_sat(i), Some(address.script_pubkey()));
            let tx = rpc.ledger.create_transaction(vec![], vec![txout]);

            rpc.ledger.add_transaction_unconditionally(tx).unwrap();
        }
        assert_eq!(
            rpc.ledger.calculate_balance().unwrap(),
            Amount::from_sat((0..100).sum())
        );

        // send_to_address should combine UTXO's and create a valid transaction.
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

        // User should have 0x45 less Sats.
        assert_eq!(
            rpc.ledger.calculate_balance().unwrap(),
            Amount::from_sat((0..100).sum::<u64>() - 0x45)
        );
    }

    #[test]
    fn get_new_address() {
        let rpc = Client::new("", bitcoincore_rpc::Auth::None).unwrap();

        let address = rpc.get_new_address(None, None).unwrap();

        assert!(address.is_valid_for_network(Network::Regtest));
        assert!(!address.is_valid_for_network(Network::Testnet));
        assert!(!address.is_valid_for_network(Network::Signet));
        assert!(!address.is_valid_for_network(Network::Bitcoin));
        assert_eq!(
            *rpc.ledger.get_credentials()[0].address.as_unchecked(),
            address
        );

        const ADDRESS_COUNT: usize = 100;
        let mut prev = address;
        for i in 0..ADDRESS_COUNT {
            let curr = rpc.get_new_address(None, None).unwrap();

            assert_ne!(prev, curr);
            assert!(curr.is_valid_for_network(Network::Regtest));
            assert!(!curr.is_valid_for_network(Network::Testnet));
            assert!(!curr.is_valid_for_network(Network::Signet));
            assert!(!curr.is_valid_for_network(Network::Bitcoin));
            assert_eq!(
                *rpc.ledger.get_credentials()[i + 1].address.as_unchecked(),
                curr
            );

            prev = curr;
        }
    }

    #[test]
    fn generate_to_address() {
        let rpc = Client::new("", bitcoincore_rpc::Auth::None).unwrap();

        let address = rpc.get_new_address(None, None).unwrap().assume_checked();

        // Empty wallet should reject transaction.
        let txout = rpc
            .ledger
            .create_txout(Amount::from_sat(1), Some(address.script_pubkey()));
        let tx = rpc.ledger.create_transaction(vec![], vec![txout]);
        if let Ok(()) = rpc.ledger.check_transaction(&tx) {
            assert!(false);
        };

        // Generating blocks should add funds to wallet.
        rpc.generate_to_address(101, &address).unwrap();

        // Wallet has funds now. It should not be rejected.
        let txin = rpc.ledger._create_txin(
            rpc.ledger
                ._get_transactions()
                .get(0)
                .unwrap()
                .compute_txid(),
            0,
        );
        let txout = rpc
            .ledger
            .create_txout(Amount::from_sat(1), Some(address.script_pubkey()));
        let tx = rpc.ledger.create_transaction(vec![txin], vec![txout]);
        if let Err(e) = rpc.ledger.check_transaction(&tx) {
            assert!(false, "{:?}", e);
        };
    }

    #[test]
    fn get_balance() {
        let rpc = Client::new("", bitcoincore_rpc::Auth::None).unwrap();

        let credential = Ledger::generate_credential_from_witness();
        rpc.ledger.add_credential(credential.clone());
        let address = credential.address;

        assert_eq!(rpc.get_balance(None, None).unwrap(), Amount::from_sat(0));

        let txout = rpc
            .ledger
            .create_txout(Amount::from_sat(0x45), Some(address.script_pubkey()));
        let tx = rpc.ledger.create_transaction(vec![], vec![txout]);
        rpc.ledger.add_transaction_unconditionally(tx).unwrap();

        assert_eq!(rpc.get_balance(None, None).unwrap(), Amount::from_sat(0x45));
    }
}
