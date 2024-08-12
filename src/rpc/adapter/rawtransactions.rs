//! # Rawtransactions RPCs

use super::{decode_from_hex, encode_to_hex};
use crate::Client;
use bitcoin::{BlockHash, Transaction, Txid};
use bitcoincore_rpc::{Error, RpcApi};

pub fn getrawtransaction(
    client: &Client,
    txid: String,
    verbose: Option<bool>,
    blockhash: Option<BlockHash>,
) -> Result<String, Error> {
    let txid = decode_from_hex::<Txid>(txid)?;

    let res: String = match verbose {
        None | Some(false) => {
            let tx = client.get_raw_transaction(&txid, blockhash.as_ref())?;
            encode_to_hex(tx)
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
    let tx = decode_from_hex::<Transaction>(hexstring)?;

    let txid = client.send_raw_transaction(&tx)?;
    let txid = encode_to_hex(txid);

    Ok(txid)
}
