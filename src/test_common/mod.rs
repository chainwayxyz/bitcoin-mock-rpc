//! # Common Utilities for Unit and Integration Tests
//!
//! This crate compiles for test targets and provides common utilities for it.
//!
//! This crate is in `src/` directory because unit tests can't access `tests/`
//! directory.

use bitcoin::{
    absolute,
    key::UntweakedPublicKey,
    opcodes::all::OP_EQUAL,
    taproot::{LeafVersion, TaprootBuilder},
    Amount, OutPoint, ScriptBuf, Transaction, TxIn, TxOut, Txid, Witness, WitnessProgram,
};
use std::str::FromStr;

#[allow(unused)]
pub fn create_txin(txid: Txid) -> TxIn {
    let secp = bitcoin::secp256k1::Secp256k1::new();
    let internal_key = UntweakedPublicKey::from(
        bitcoin::secp256k1::PublicKey::from_str(
            "0250929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0",
        )
        .unwrap(),
    );

    let mut script = ScriptBuf::new();
    script.push_slice([12, 34]);
    script.push_instruction(bitcoin::script::Instruction::Op(OP_EQUAL));

    let taproot_builder = TaprootBuilder::new().add_leaf(0, script.clone()).unwrap();
    let taproot_spend_info = taproot_builder.finalize(&secp, internal_key).unwrap();

    let witness_program =
        WitnessProgram::p2tr(&secp, internal_key, taproot_spend_info.merkle_root());

    let mut control_block_bytes = Vec::new();
    taproot_spend_info
        .control_block(&(script.clone(), LeafVersion::TapScript))
        .unwrap()
        .encode(&mut control_block_bytes)
        .unwrap();

    let mut script2 = ScriptBuf::new();
    script2.push_slice([12, 34]);
    let mut witness = Witness::new();
    witness.push(script2);
    witness.push(script.to_bytes());
    witness.push(control_block_bytes);

    TxIn {
        previous_output: OutPoint { txid, vout: 0 },
        witness,
        ..Default::default()
    }
}

#[allow(unused)]
pub fn create_txout(satoshi: u64, script_pubkey: Option<ScriptBuf>) -> TxOut {
    TxOut {
        value: Amount::from_sat(satoshi),
        script_pubkey: match script_pubkey {
            Some(script_pubkey) => script_pubkey,
            None => ScriptBuf::new(),
        },
    }
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
