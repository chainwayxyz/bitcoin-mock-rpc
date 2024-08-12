//! # RPC Adapter Interface
//!
//! This crate provides an adapter interface that aims to mimic real Bitcoin
//! RPC interface. It builds on [`RpcApi`].

mod blockchain;

use bitcoin::hex::DisplayHex;
pub use blockchain::*;

fn encode_to_hex<T>(strct: T) -> String
where
    T: bitcoin::consensus::Encodable,
{
    let mut encoded: Vec<u8> = Vec::new();
    strct.consensus_encode(&mut encoded).unwrap();

    encoded.to_hex_string(bitcoin::hex::Case::Upper)
}
fn decode_from_hex<T>(hex: String) -> Result<T, bitcoincore_rpc::Error>
where
    T: bitcoin::consensus::Decodable,
{
    let mut hex = hex.as_bytes();
    match T::consensus_decode(&mut hex) {
        Ok(t) => Ok(t),
        Err(e) => Err(encode_decode_to_rpc(e)),
    }
}

fn encode_decode_to_rpc(error: bitcoin::consensus::encode::Error) -> bitcoincore_rpc::Error {
    bitcoincore_rpc::Error::BitcoinSerialization(bitcoin::consensus::encode::FromHexError::Decode(
        bitcoin::consensus::DecodeError::Consensus(error),
    ))
}
