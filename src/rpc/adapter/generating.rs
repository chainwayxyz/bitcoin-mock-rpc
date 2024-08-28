//! # Generating RPCs

use crate::{utils::encode_to_hex, Client};
use bitcoin::Address;
use bitcoincore_rpc::{Error, RpcApi};
use std::str::FromStr;

pub fn generatetoaddress(
    client: &Client,
    nblocks: usize,
    address: String,
    _maxtries: Option<usize>,
) -> Result<Vec<String>, Error> {
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

    tracing::trace!("Address converted: {address}");

    let hashes = client.generate_to_address(nblocks as u64, &address)?;
    let hashes: Vec<String> = hashes.iter().map(encode_to_hex).collect();

    Ok(hashes)
}

#[cfg(test)]
mod tests {
    use crate::{Client, RpcApiWrapper};
    use bitcoincore_rpc::RpcApi;

    #[test]
    fn generatetoaddress() {
        let client = Client::new("generatetoaddress", bitcoincore_rpc::Auth::None).unwrap();

        assert_eq!(client.get_block_count().unwrap(), 0);

        let address = client.get_new_address(None, None).unwrap().assume_checked();
        super::generatetoaddress(&client, 101, address.to_string(), None).unwrap();

        assert_eq!(client.get_block_count().unwrap(), 101);
    }
}
