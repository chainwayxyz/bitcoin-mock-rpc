//! # Generating RPCs

use crate::Client;
use bitcoin::Address;
use bitcoincore_rpc::{Error, RpcApi};
use std::str::FromStr;

pub fn generatetoaddress(
    client: &Client,
    nblocks: usize,
    address: String,
    _maxtries: usize,
) -> Result<String, Error> {
    let address = match Address::from_str(&address) {
        Ok(a) => a,
        Err(_e) => {
            return Err(bitcoincore_rpc::Error::BitcoinSerialization(
                bitcoin::consensus::encode::FromHexError::Decode(
                    bitcoin::consensus::DecodeError::TooManyBytes, // TODO: Return the actual error.
                ),
            ));
        }
    }
    .assume_checked();

    let hashes = client.generate_to_address(nblocks as u64, &address)?;

    Ok(serde_json::to_string_pretty(&hashes)?)
}
