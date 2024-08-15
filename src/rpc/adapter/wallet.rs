//! # Wallet RPCs

use super::{decode_from_hex, encode_to_hex};
use crate::Client;
use bitcoin::{Address, Amount, Txid};
use bitcoincore_rpc::{json, Error, RpcApi};
use std::str::FromStr;

pub fn getnewaddress(
    client: &Client,
    label: Option<String>,
    address_type: Option<String>,
) -> Result<String, Error> {
    let address_type = match address_type {
        Some(a) => Some(serde_json::from_str::<json::AddressType>(&a)?),
        None => None,
    };

    let res = client.get_new_address(label.as_deref(), address_type)?;

    Ok(res.assume_checked().to_string())
}

pub fn gettransaction(
    client: &Client,
    txid: String,
    include_watchonly: Option<bool>,
    _verbose: Option<bool>,
) -> Result<String, Error> {
    let txid = decode_from_hex::<Txid>(txid)?;

    let tx = client.get_transaction(&txid, include_watchonly)?;

    Ok(serde_json::to_string_pretty(&tx)?)
}

// This has nothing to do with us. Ignore it.
#[allow(clippy::too_many_arguments)]
pub fn sendtoaddress(
    client: &Client,
    address: String,
    amount: String,
    comment: Option<&str>,
    comment_to: Option<&str>,
    subtractfeefromamount: Option<bool>,
    replaceable: Option<bool>,
    conf_target: Option<u32>,
    _estimate_mode: Option<&str>,
    _avoid_reuse: Option<bool>,
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
    let amount = match Amount::from_str(&amount) {
        Ok(a) => a,
        Err(_e) => {
            return Err(bitcoincore_rpc::Error::BitcoinSerialization(
                bitcoin::consensus::encode::FromHexError::Decode(
                    bitcoin::consensus::DecodeError::TooManyBytes, // TODO: Return the actual error.
                ),
            ));
        }
    };

    let txid = client.send_to_address(
        &address,
        amount,
        comment,
        comment_to,
        subtractfeefromamount,
        replaceable,
        conf_target,
        None,
    )?;

    Ok(encode_to_hex::<Txid>(txid))
}

#[cfg(test)]
mod tests {
    use crate::{Client, RpcApiWrapper};
    use bitcoin::Address;
    use std::str::FromStr;

    #[test]
    fn getnewaddress() {
        let client = Client::new("getnewaddress", bitcoincore_rpc::Auth::None).unwrap();

        let address = super::getnewaddress(&client, None, None).unwrap();
        let _should_not_panic = Address::from_str(&address).unwrap();
    }
}
