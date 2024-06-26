//! Transaction related integration tests.

use bitcoin::{Amount, OutPoint, TxIn};
use bitcoin_mock_rpc::{Client, RpcApiWrapper};
use bitcoincore_rpc::{Auth, RpcApi};
use std::thread;

mod common;
use common::test_common;

#[test]
#[ignore = "Not necessary after the send_to_address simplification"]
fn send_to_address_multi_threaded() {
    // Bacause `thread::spawn` moves value to closure, cloning a new is needed. This is good,
    // because cloning an rpc struct should have a persistent ledger even though there are more than
    // one accessors.
    let rpc = Client::new("", Auth::None).unwrap();
    let cloned_rpc = rpc.clone();
    let address = rpc.get_new_address(None, None).unwrap().assume_checked();
    let deposit_address = test_common::create_address_from_witness();
    let cloned_deposit_address = deposit_address.clone();

    rpc.generate_to_address(101, &address).unwrap();
    let initial_balance = rpc.get_balance(None, None).unwrap();
    let deposit_value = initial_balance / 4;

    thread::spawn(move || {
        cloned_rpc
            .send_to_address(
                &cloned_deposit_address,
                deposit_value,
                None,
                None,
                None,
                None,
                None,
                None,
            )
            .unwrap();

        assert_eq!(
            cloned_rpc.get_balance(None, None).unwrap(),
            initial_balance - deposit_value
        );
    })
    .join()
    .unwrap();

    // Change made in other rpc connection should be available now.
    assert_eq!(
        rpc.get_balance(None, None).unwrap(),
        initial_balance - deposit_value
    );

    // Adding new blocks should add more funds.
    rpc.send_to_address(
        &deposit_address,
        deposit_value,
        None,
        None,
        None,
        None,
        None,
        None,
    )
    .unwrap();
    assert_eq!(
        rpc.get_balance(None, None).unwrap(),
        initial_balance - deposit_value - deposit_value
    ); // No multiplication over `Amount`.
    assert!(rpc.get_balance(None, None).unwrap() < initial_balance);
}

#[test]
fn use_utxo_from_send_to_address() {
    let rpc = Client::new("", Auth::None).unwrap();

    let address = rpc.get_new_address(None, None).unwrap().assume_checked();
    let deposit_address = test_common::create_address_from_witness();

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
    assert_eq!(rpc.get_balance(None, None).unwrap(), deposit_value * 0x1F);

    let tx = rpc.get_raw_transaction(&txid, None).unwrap();
    assert_eq!(tx.output.get(0).unwrap().value, deposit_value * 0x1F);

    // Valid tx.
    let txin = TxIn {
        previous_output: OutPoint { txid, vout: 0 },
        ..Default::default()
    };
    let txout = test_common::create_txout(0x45, Some(deposit_address.script_pubkey()));
    let tx = test_common::create_transaction(vec![txin.clone()], vec![txout]);
    rpc.send_raw_transaction(&tx).unwrap();

    // Invalid tx.
    let txout = test_common::create_txout(0x45 * 0x45, Some(deposit_address.script_pubkey()));
    let tx = test_common::create_transaction(vec![txin], vec![txout]);
    if let Ok(_) = rpc.send_raw_transaction(&tx) {
        assert!(false);
    };
}
