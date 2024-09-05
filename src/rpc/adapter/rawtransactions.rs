//! # Rawtransactions RPCs

use crate::utils::encode_to_hex;
use crate::Client;
use bitcoin::{consensus::encode::deserialize_hex, hex::DisplayHex, BlockHash, Transaction, Txid};
use bitcoincore_rpc::{Error, RpcApi};
use serde::{ser::SerializeStruct, Deserialize, Serialize, Serializer};
use std::str::FromStr;

#[derive(Clone, PartialEq, Eq, Debug, Deserialize)]
pub enum GetrawtransactionReturn {
    NoneVerbose(String),
    Verbose(Box<bitcoincore_rpc::json::GetRawTransactionResult>),
}
impl Serialize for GetrawtransactionReturn {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            GetrawtransactionReturn::NoneVerbose(stri) => serializer.serialize_str(stri),
            GetrawtransactionReturn::Verbose(strct) => {
                let mut state = serializer.serialize_struct("GetRawTransactionResult", 14)?;

                if strct.in_active_chain.is_some() {
                    state.serialize_field("in_active_chain", &strct.in_active_chain)?;
                }
                state
                    .serialize_field("hex", &strct.hex.to_hex_string(bitcoin::hex::Case::Lower))?;
                state.serialize_field("txid", &strct.txid)?;
                state.serialize_field("hash", &strct.hash)?;
                state.serialize_field("size", &strct.size)?;
                state.serialize_field("vsize", &strct.vsize)?;
                state.serialize_field("version", &strct.version)?;
                state.serialize_field("locktime", &strct.locktime)?;

                #[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
                struct Vin {
                    sequence: u32,
                }
                let vins: Vec<Vin> = strct
                    .vin
                    .iter()
                    .map(|vin| Vin {
                        sequence: vin.sequence,
                    })
                    .collect();
                state.serialize_field("vin", &vins)?;

                state.serialize_field("vout", &strct.vout)?;
                if strct.blockhash.is_some() {
                    state.serialize_field("blockhash", &strct.blockhash)?;
                }
                if strct.confirmations.is_some() {
                    state.serialize_field("confirmations", &strct.confirmations)?;
                }
                if strct.time.is_some() {
                    state.serialize_field("time", &strct.time)?;
                }
                if strct.blocktime.is_some() {
                    state.serialize_field("blocktime", &strct.blocktime)?;
                }
                state.end()
            }
        }
    }
}
pub fn getrawtransaction(
    client: &Client,
    txid: String,
    verbose: Option<bool>,
    blockhash: Option<BlockHash>,
) -> Result<GetrawtransactionReturn, Error> {
    let txid = Txid::from_str(&txid).unwrap();

    let res: GetrawtransactionReturn = match verbose {
        None | Some(false) => {
            let tx = client.get_raw_transaction(&txid, blockhash.as_ref())?;

            GetrawtransactionReturn::NoneVerbose(encode_to_hex(&tx))
        }
        Some(true) => {
            let tx: bitcoincore_rpc::json::GetRawTransactionResult =
                client.get_raw_transaction_info(&txid, blockhash.as_ref())?;

            GetrawtransactionReturn::Verbose(Box::new(tx))
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
    client: &Client,
    hexstring: String,
    _options: Option<String>,
    iswitness: Option<bool>,
) -> Result<bitcoincore_rpc::json::FundRawTransactionResult, Error> {
    let tx = deserialize_hex::<Transaction>(&hexstring).unwrap();

    client.fund_raw_transaction(&tx, None, iswitness)
}

pub fn signrawtransactionwithwallet(
    client: &Client,
    hexstring: String,
    _prevtxs: Option<String>,
    _sighashtype: Option<String>,
) -> Result<bitcoincore_rpc::json::SignRawTransactionResult, Error> {
    let tx = deserialize_hex::<Transaction>(&hexstring).unwrap();

    client.sign_raw_transaction_with_wallet(&tx, None, None)
}

#[cfg(test)]
mod tests {
    use crate::{
        ledger,
        rpc::adapter::GetrawtransactionReturn,
        utils::{decode_from_hex, encode_to_hex},
        Client, RpcApiWrapper,
    };
    use bitcoin::{
        absolute::LockTime, consensus::Decodable, transaction::Version, Amount, OutPoint,
        Transaction, TxIn, TxOut, Txid,
    };
    use bitcoincore_rpc::RpcApi;

    #[test]
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

        let encoded_tx = super::getrawtransaction(&client, txid.to_string(), None, None).unwrap();

        if let GetrawtransactionReturn::NoneVerbose(encoded_tx) = encoded_tx {
            assert_eq!(tx, decode_from_hex(encoded_tx).unwrap());
        } else {
            panic!("");
        }
    }

    #[test]
    fn getrawtransactionverbose() {
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
            super::getrawtransaction(&client, txid.to_string(), Some(true), None).unwrap();

        if let GetrawtransactionReturn::Verbose(encoded_tx) = encoded_tx {
            assert_eq!(
                tx,
                Transaction::consensus_decode(&mut encoded_tx.hex.as_slice()).unwrap()
            );
        } else {
            panic!("Should be verbose variant");
        }
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
