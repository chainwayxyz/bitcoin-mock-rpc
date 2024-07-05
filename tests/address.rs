//! Address related integration tests.

use bitcoin_mock_rpc::{Client, RpcApiWrapper};
use bitcoincore_rpc::{Auth, RpcApi};
use std::thread;

#[test]
#[ignore = "Not implemented yet"]
fn generate_to_address() {
    let rpc = Client::new("", Auth::None).unwrap();
    let address = rpc.get_new_address(None, None).unwrap().assume_checked();

    let initial_balance = rpc.get_balance(None, None).unwrap();

    rpc.generate_to_address(101, &address).unwrap();
    assert!(rpc.get_balance(None, None).unwrap() > initial_balance);
}

#[test]
#[ignore = "Not implemented yet"]
fn generate_to_address_multi_threaded() {
    let rpc = Client::new("", Auth::None).unwrap();
    let cloned_rpc = rpc.clone();
    let address = rpc.get_new_address(None, None).unwrap().assume_checked();
    let cloned_address = address.clone();

    let initial_balance = rpc.get_balance(None, None).unwrap();

    thread::spawn(move || {
        cloned_rpc
            .generate_to_address(101, &cloned_address)
            .unwrap();

        assert!(cloned_rpc.get_balance(None, None).unwrap() > initial_balance);
    })
    .join()
    .unwrap();

    // Change made in other rpc connection should be available now.
    let changed_balance = rpc.get_balance(None, None).unwrap();
    assert!(changed_balance > initial_balance);

    // Adding new blocks should add more funds.
    rpc.generate_to_address(101, &address).unwrap();
    assert!(rpc.get_balance(None, None).unwrap() > changed_balance);
    assert!(rpc.get_balance(None, None).unwrap() > initial_balance);
}
