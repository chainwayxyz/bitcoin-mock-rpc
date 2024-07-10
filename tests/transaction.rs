//! Transaction related integration tests.

use bitcoin::{Amount, OutPoint, TxIn};
use bitcoin_mock_rpc::{Client, RpcApiWrapper};
use bitcoincore_rpc::{Auth, RpcApi};
use std::thread;

mod common;

#[test]
fn send_to_address_multi_threaded() {
    let rpc = Client::new("send_to_address_multi_threaded", Auth::None).unwrap();
    let cloned_rpc = rpc.clone();
    let witness = common::create_witness();
    let address = common::create_address_from_witness(witness.0);

    thread::spawn(move || {
        cloned_rpc
            .send_to_address(
                &address,
                Amount::from_sat(0x45),
                None,
                None,
                None,
                None,
                None,
                None,
            )
            .unwrap();

        // assert_eq!(
        //     cloned_rpc.get_balance(None, None).unwrap(),
        //     Amount::from_sat(0x45)
        // );
    })
    .join()
    .unwrap();

    // Change made in other rpc connection should also be available here.
    // assert_eq!(rpc.get_balance(None, None).unwrap(), Amount::from_sat(0x45));
}

#[test]
fn use_utxo_from_send_to_address() {
    let rpc = Client::new("use_utxo_from_send_to_address", Auth::None).unwrap();

    let witness = common::create_witness();
    let address = common::create_address_from_witness(witness.0);
    let witness2 = common::create_witness();
    let deposit_address = common::create_address_from_witness(witness2.0);

    let deposit_value = Amount::from_sat(0x45);

    let txid = rpc
        .send_to_address(
            &address,
            deposit_value * 0x1F,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .unwrap();
    // assert_eq!(rpc.get_balance(None, None).unwrap(), deposit_value * 0x1F);

    let tx = rpc.get_raw_transaction(&txid, None).unwrap();
    assert_eq!(tx.output.get(0).unwrap().value, deposit_value * 0x1F);

    // Valid tx.
    let txin = TxIn {
        previous_output: OutPoint { txid, vout: 0 },
        witness: witness.1,
        ..Default::default()
    };
    let txout = common::create_txout(Amount::from_sat(0x45), deposit_address.script_pubkey());
    let tx = common::create_transaction(vec![txin.clone()], vec![txout]);
    rpc.send_raw_transaction(&tx).unwrap();

    // Invalid tx.
    let txout = common::create_txout(
        Amount::from_sat(0x45 * 0x45),
        deposit_address.script_pubkey(),
    );
    let tx = common::create_transaction(vec![txin], vec![txout]);
    if let Ok(_) = rpc.send_raw_transaction(&tx) {
        assert!(false);
    };
}
