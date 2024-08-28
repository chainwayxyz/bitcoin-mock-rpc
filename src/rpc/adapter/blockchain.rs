//! # Blockchain RPCs

use crate::utils::{decode_from_hex, encode_to_hex};
use crate::Client;
use bitcoin::{BlockHash, Txid};
use bitcoincore_rpc::{json, Error, RpcApi};
use std::str::FromStr;

pub fn getbestblockhash(client: &Client) -> Result<String, Error> {
    let res = client.get_best_block_hash()?;

    Ok(encode_to_hex(&res))
}

#[tracing::instrument]
pub fn getblock(
    client: &Client,
    blockhash: String,
    verbosity: Option<usize>,
) -> Result<String, Error> {
    let blockhash = decode_from_hex::<BlockHash>(blockhash)?;
    tracing::trace!("Decoded block hash: {blockhash:?}");

    let block = client.get_block(&blockhash)?;
    let encoded = encode_to_hex(&block);
    tracing::trace!("Block: {block:?}");
    tracing::trace!("Encoded value: {encoded}");

    match verbosity {
        Some(0) => Ok(encoded),
        None | Some(1) => Ok(serde_json::to_string(&block)?),
        _ => Err(Error::UnexpectedStructure),
    }
}

pub fn getblockcount(client: &Client) -> Result<usize, Error> {
    Ok(client.get_block_count()? as usize)
}

pub fn getblockhash(client: &Client, height: usize) -> Result<String, Error> {
    let block_hash = client.get_block_hash(height as u64)?;

    Ok(encode_to_hex(&block_hash))
}

pub fn getblockheader(
    client: &Client,
    blockhash: String,
    verbose: Option<bool>,
) -> Result<String, Error> {
    let blockhash = decode_from_hex::<BlockHash>(blockhash)?;
    let header = client.get_block_header(&blockhash)?;

    match verbose {
        None | Some(true) => Ok(serde_json::to_string(&header).unwrap()),
        Some(false) => Ok(encode_to_hex(&header)),
    }
}

pub fn gettxout(
    client: &Client,
    txid: String,
    n: u32,
    include_mempool: Option<bool>,
) -> Result<json::GetTxOutResult, Error> {
    let txid = match Txid::from_str(&txid) {
        Ok(txid) => txid,
        Err(e) => return Err(Error::ReturnedError(e.to_string())),
    };

    let txout = client.get_tx_out(&txid, n, include_mempool)?;

    match txout {
        Some(to) => Ok(to),
        None => Err(Error::UnexpectedStructure),
    }
}

#[cfg(test)]
mod tests {
    use crate::{Client, RpcApiWrapper};
    use bitcoin::consensus::Decodable;
    use bitcoin::BlockHash;
    use bitcoincore_rpc::RpcApi;

    #[test]
    fn getbestblockhash() {
        let client = Client::new("getbestblockhash", bitcoincore_rpc::Auth::None).unwrap();

        // No blocks created, no blocks are available to return.
        assert!(super::getbestblockhash(&client).is_err());

        let address = client.get_new_address(None, None).unwrap().assume_checked();
        client.generate_to_address(101, &address).unwrap();

        let block = super::getbestblockhash(&client).unwrap();
        let hash = BlockHash::consensus_decode(&mut block.as_bytes()).unwrap();
        println!("Block hash: {:?}", hash);
    }

    #[test]
    fn getblockcount() {
        let client = Client::new("getblockcount", bitcoincore_rpc::Auth::None).unwrap();

        assert_eq!(super::getblockcount(&client).unwrap(), 0);

        let address = client.get_new_address(None, None).unwrap().assume_checked();
        client.generate_to_address(101, &address).unwrap();

        assert_eq!(super::getblockcount(&client).unwrap(), 101);
    }
}
