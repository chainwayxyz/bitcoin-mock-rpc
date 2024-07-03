use bitcoin::{
    absolute,
    key::UntweakedPublicKey,
    opcodes::OP_TRUE,
    taproot::{LeafVersion, TaprootBuilder},
    Address, Amount, OutPoint, ScriptBuf, Transaction, TxIn, TxOut, Txid, Witness, WitnessProgram,
};
use bitcoin_mock_rpc::Client;
use bitcoincore_rpc::RpcApi;
use std::str::FromStr;

pub async fn send_raw_transaction_async(rpc: Client, tx: Transaction) {
    rpc.send_raw_transaction(&tx).unwrap();
}

pub fn create_witness() -> (WitnessProgram, Witness) {
    let secp = bitcoin::secp256k1::Secp256k1::new();
    let internal_key = UntweakedPublicKey::from(
        bitcoin::secp256k1::PublicKey::from_str(
            "0250929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0",
        )
        .unwrap(),
    );

    let mut script = ScriptBuf::new();
    script.push_instruction(bitcoin::script::Instruction::Op(OP_TRUE));

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

    let mut witness = Witness::new();
    witness.push(script.to_bytes());
    witness.push(control_block_bytes);

    (witness_program, witness)
}

pub fn create_address_from_witness() -> Address {
    let witness_program = create_witness().0;

    Address::from_witness_program(witness_program, bitcoin::Network::Regtest)
}

pub fn create_txin(txid: Txid, vout: u32) -> TxIn {
    TxIn {
        previous_output: OutPoint { txid, vout },
        ..Default::default()
    }
}

pub fn create_txout(value: Amount, script_pubkey: ScriptBuf) -> TxOut {
    TxOut {
        value,
        script_pubkey,
    }
}

pub fn create_transaction(tx_ins: Vec<TxIn>, tx_outs: Vec<TxOut>) -> Transaction {
    bitcoin::Transaction {
        version: bitcoin::transaction::Version(2),
        lock_time: absolute::LockTime::from_consensus(0),
        input: tx_ins,
        output: tx_outs,
    }
}
