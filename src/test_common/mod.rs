//! # Common Utilities for Unit and Integration Tests
//!
//! This crate compiles for test targets and provides common utilities for it.
//!
//! This crate is in `src/` directory because unit tests can't access `tests/`
//! directory.

use bitcoin::{absolute, Address, Network, Transaction, TxIn, TxOut, XOnlyPublicKey};
use secp256k1::Secp256k1;

#[allow(unused)]
pub fn get_temp_address() -> Address {
    let secp = Secp256k1::new();
    let xonly_public_key = XOnlyPublicKey::from_slice(&[
        0x78u8, 0x19u8, 0x90u8, 0xd7u8, 0xe2u8, 0x11u8, 0x8cu8, 0xc3u8, 0x61u8, 0xa9u8, 0x3au8,
        0x6fu8, 0xccu8, 0x54u8, 0xceu8, 0x61u8, 0x1du8, 0x6du8, 0xf3u8, 0x81u8, 0x68u8, 0xd6u8,
        0xb1u8, 0xedu8, 0xfbu8, 0x55u8, 0x65u8, 0x35u8, 0xf2u8, 0x20u8, 0x0cu8, 0x4b,
    ])
    .unwrap();

    Address::p2tr(&secp, xonly_public_key, None, Network::Regtest)
}

#[allow(unused)]
pub fn create_transaction(tx_ins: Vec<TxIn>, tx_outs: Vec<TxOut>) -> Transaction {
    bitcoin::Transaction {
        version: bitcoin::transaction::Version(2),
        lock_time: absolute::LockTime::from_consensus(0),
        input: tx_ins,
        output: tx_outs,
    }
}
