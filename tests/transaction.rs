//! Transaction related integration tests.

use bitcoin_mock_rpc::{Client, RpcApiWrapper};
use bitcoincore_rpc::{Auth, RpcApi};
use std::thread;

mod common;
use common::test_common;

#[test]
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
