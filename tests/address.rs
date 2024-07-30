//! Address related integration tests.

use bitcoin::Amount;
use bitcoin_mock_rpc::{Client, RpcApiWrapper};
use bitcoincore_rpc::{Auth, RpcApi};
use std::thread;

#[test]
fn generate_to_address() {
    let rpc = Client::new("generate_to_address", Auth::None).unwrap();
    let address = rpc.get_new_address(None, None).unwrap().assume_checked();

    // Should increase block height.
    rpc.generate_to_address(1, &address).unwrap();

    let txid = rpc
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

    rpc.generate_to_address(101, &address).unwrap();

    let details = rpc.get_transaction(&txid, None).unwrap();

    assert_eq!(details.info.blockheight, Some(102));
    assert_eq!(details.info.confirmations, 101);
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

        let txid = cloned_rpc
            .send_to_address(
                &cloned_address,
                Amount::from_sat(0x45),
                None,
                None,
                None,
                None,
                None,
                None,
            )
            .unwrap();

        cloned_rpc
            .generate_to_address(101, &cloned_address)
            .unwrap();

        let details = cloned_rpc.get_transaction(&txid, None).unwrap();

        assert_eq!(details.info.blockheight, Some(202));
        assert_eq!(details.info.confirmations, 101);
    })
    .join()
    .unwrap();

    // Adding new blocks should increase block height.
    rpc.generate_to_address(101, &address).unwrap();

    let txid = rpc
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

    let details = rpc.get_transaction(&txid, None).unwrap();

    assert_eq!(details.info.blockheight, Some(303));
    assert_eq!(details.info.confirmations, 0);
}
