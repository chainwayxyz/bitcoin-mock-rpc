//! # RPC Adapter Interface
//!
//! This crate provides an adapter interface that aims to mimic real Bitcoin
//! RPC interface.

use bitcoin::consensus::encode::{deserialize_hex, serialize_hex};

mod blockchain;
mod generating;
mod rawtransactions;
mod wallet;

pub use blockchain::*;
pub use generating::*;
pub use rawtransactions::*;
pub use wallet::*;

/// Encodes given Rust struct to hex string.
fn encode_to_hex<T>(strct: T) -> String
where
    T: bitcoin::consensus::Encodable,
{
    serialize_hex::<T>(&strct)
}

/// Decodes given hex string to a Rust struct.
fn decode_from_hex<T>(hex: String) -> Result<T, bitcoincore_rpc::Error>
where
    T: bitcoin::consensus::Decodable,
{
    Ok(deserialize_hex::<T>(&hex)?)
}

fn encode_decode_to_rpc_error(error: bitcoin::consensus::encode::Error) -> bitcoincore_rpc::Error {
    bitcoincore_rpc::Error::BitcoinSerialization(bitcoin::consensus::encode::FromHexError::Decode(
        bitcoin::consensus::DecodeError::Consensus(error),
    ))
}

#[cfg(test)]
mod tests {
    use super::{decode_from_hex, encode_to_hex};
    use bitcoin::{hashes::sha256d::Hash, Txid};
    use std::str::FromStr;

    #[test]
    fn encode_decode_txid() {
        let txid = Txid::from_raw_hash(
            Hash::from_str("e6d467860551868fe599889ea9e622ae1ff08891049e934f83a783a3ea5fbc12")
                .unwrap(),
        );

        let encoded_txid = encode_to_hex(txid);
        let decoded_txid = decode_from_hex::<Txid>(encoded_txid).unwrap();

        assert_eq!(txid, decoded_txid);
    }
}
