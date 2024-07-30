//! Address related integration tests.

use bitcoin_mock_rpc::{Client, RpcApiWrapper};
use bitcoincore_rpc::{Auth, RpcApi};
use std::thread;

#[test]
fn generate_to_address() {
    let rpc = Client::new("generate_to_address", Auth::None).unwrap();
    let address = rpc.get_new_address(None, None).unwrap().assume_checked();

    // Should increase block height.
    rpc.generate_to_address(101, &address).unwrap();
}

#[test]
fn generate_to_address_multi_threaded() {
    let rpc = Client::new("generate_to_address_multi_threaded", Auth::None).unwrap();
    let cloned_rpc = rpc.clone();
    let address = rpc.get_new_address(None, None).unwrap().assume_checked();
    let cloned_address = address.clone();

    thread::spawn(move || {
        cloned_rpc
            .generate_to_address(101, &cloned_address)
            .unwrap();
    })
    .join()
    .unwrap();

    // Adding new blocks should increase block height.
    rpc.generate_to_address(101, &address).unwrap();
}
