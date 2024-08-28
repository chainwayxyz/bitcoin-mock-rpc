//! # Rawtransactions RPCs

use crate::utils::encode_to_hex;
use crate::Client;
use bitcoin::{BlockHash, Txid};
use bitcoincore_rpc::{Error, RpcApi};
use std::str::FromStr;

pub fn getrawtransaction(
    client: &Client,
    txid: String,
    verbose: Option<bool>,
    blockhash: Option<BlockHash>,
) -> Result<String, Error> {
    let txid = Txid::from_str(&txid).unwrap();

    let res: String = match verbose {
        None | Some(false) => {
            let tx = client.get_raw_transaction(&txid, blockhash.as_ref())?;

            encode_to_hex(&tx)
        }
        Some(true) => {
            let tx = client.get_raw_transaction_info(&txid, blockhash.as_ref())?;

            serde_json::to_string_pretty(&tx).unwrap()
        }
    };

    Ok(res)
}

pub fn sendrawtransaction(
    client: &Client,
    hexstring: String,
    _maxfeerate: Option<usize>,
) -> Result<String, Error> {
    let txid = client.send_raw_transaction(hexstring)?;
    let txid = encode_to_hex(&txid);

    Ok(txid)
}

pub fn fundrawtransaction(
    _client: &Client,
    _hexstring: String,
    _options: Option<String>,
    _iswitness: Option<bool>,
) -> Result<String, Error> {
    todo!()
}

pub fn signrawtransactionwithwallet(
    _client: &Client,
    _hexstring: String,
    _prevtxs: Option<String>,
    _sighashtype: Option<bool>,
) -> Result<String, Error> {
    todo!()
}

#[cfg(test)]
mod tests {
    use crate::{
        ledger,
        utils::{decode_from_hex, encode_to_hex},
        Client, RpcApiWrapper,
    };
    use bitcoin::{
        absolute::LockTime, transaction::Version, Amount, OutPoint, Transaction, TxIn, TxOut, Txid,
    };
    use bitcoincore_rpc::RpcApi;

    #[test]
    #[ignore = "reason"]
    fn getrawtransaction() {
        let client = Client::new("getrawtransaction", bitcoincore_rpc::Auth::None).unwrap();

        let address = client.get_new_address(None, None).unwrap().assume_checked();
        let txid = client
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

        let tx = client.get_raw_transaction(&txid, None).unwrap();

        let encoded_tx =
            super::getrawtransaction(&client, encode_to_hex(&txid), None, None).unwrap();
        let encoded_tx = decode_from_hex(encoded_tx).unwrap();

        assert_eq!(tx, encoded_tx);
    }

    #[test]
    fn sendrawtransaction() {
        let client = Client::new("sendrawtransaction", bitcoincore_rpc::Auth::None).unwrap();

        let credential = ledger::Ledger::generate_credential_from_witness();

        let txid = client
            .send_to_address(
                &credential.address,
                Amount::from_sat(0x45),
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
            witness: credential.witness.unwrap(),
            ..Default::default()
        };
        let txout = TxOut {
            value: Amount::from_sat(0x1F),
            script_pubkey: credential.address.script_pubkey(),
        };
        let tx = Transaction {
            input: vec![txin],
            output: vec![txout],
            version: Version::TWO,
            lock_time: LockTime::ZERO,
        };

        let txid = super::sendrawtransaction(&client, encode_to_hex(&tx.clone()), None).unwrap();
        let txid = decode_from_hex::<Txid>(txid).unwrap();

        let read_tx = client.get_raw_transaction(&txid, None).unwrap();

        assert_eq!(tx, read_tx);
    }
}
