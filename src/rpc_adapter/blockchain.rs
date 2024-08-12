//! # Blockchain RPCs

use super::{decode_from_hex, encode_decode_to_rpc_error, encode_to_hex};
use crate::Client;
use bitcoin::{consensus::Decodable, BlockHash};
use bitcoincore_rpc::{Error, RpcApi};

pub fn getbestblockhash(client: &Client) -> Result<String, Error> {
    let res = client.get_best_block_hash()?;

    Ok(res.to_string())
}

pub fn getblock(
    client: &Client,
    blockhash: String,
    verbosity: Option<usize>,
) -> Result<String, Error> {
    let mut blockhash = blockhash.as_bytes();
    let blockhash = match BlockHash::consensus_decode(&mut blockhash) {
        Ok(bh) => bh,
        Err(e) => return Err(encode_decode_to_rpc_error(e)),
    };

    let res = client.get_block(&blockhash)?;
    let encoded = encode_to_hex(res);

    match verbosity {
        None | Some(1) => Ok(encoded),
        _ => Err(Error::UnexpectedStructure),
    }
}

pub fn getblockchaininfo(_client: &Client) -> Result<String, Error> {
    Err(Error::UnexpectedStructure)
}

pub fn getblockcount(client: &Client) -> Result<usize, Error> {
    Ok(client.get_block_count()? as usize)
}

pub fn getblockfilter(_client: &Client, _blockhash: String) -> Result<usize, Error> {
    Err(Error::UnexpectedStructure)
}

pub fn getblockhash(client: &Client, height: usize) -> Result<String, Error> {
    let block_hash = client.get_block_hash(height as u64)?;

    Ok(encode_to_hex(block_hash))
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
        Some(false) => Ok(encode_to_hex(header)),
    }
}
