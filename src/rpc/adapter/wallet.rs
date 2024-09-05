//! # Wallet RPCs

use crate::Client;
use bitcoin::{Address, Amount, Txid};
use bitcoincore_rpc::{
    json::{self, GetTransactionResult},
    Error, RpcApi,
};
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
) -> Result<GetTransactionResult, Error> {
    let txid = Txid::from_str(&txid).unwrap();

    let tx = client.get_transaction(&txid, include_watchonly)?;

    Ok(tx)
}

// This has nothing to do with us. Ignore it.
#[allow(clippy::too_many_arguments)]
pub fn sendtoaddress(
    client: &Client,
    address: String,
    amount: f64,
    comment: Option<&str>,
    comment_to: Option<&str>,
    subtractfeefromamount: Option<bool>,
    replaceable: Option<bool>,
    conf_target: Option<u32>,
    _estimate_mode: Option<&str>,
    _avoid_reuse: Option<bool>,
) -> Result<Txid, Error> {
    let address = match Address::from_str(&address) {
        Ok(a) => a,
        Err(e) => {
            return Err(bitcoincore_rpc::Error::ReturnedError(e.to_string()));
        }
    }
    .assume_checked();
    let amount = match Amount::from_float_in(amount, bitcoin::Denomination::Bitcoin) {
        Ok(a) => a,
        Err(e) => {
            return Err(bitcoincore_rpc::Error::InvalidAmount(e));
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

    Ok(txid)
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
