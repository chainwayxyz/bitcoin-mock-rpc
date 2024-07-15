//! # Block Management Related Integration Tests

use bitcoin::Amount;
use bitcoin_mock_rpc::{Client, RpcApiWrapper};
use bitcoincore_rpc::RpcApi;

mod common;

#[test]
fn mempool() {
    let rpc = Client::new("mempool", bitcoincore_rpc::Auth::None).unwrap();

    let witness = common::create_witness();
    let address = common::create_address_from_witness(witness.0);

    // Generate funds to user.
    let txid = rpc
        .send_to_address(
            &address,
            Amount::from_sat(0x45 * 0x45),
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .unwrap();

    let tx = rpc.get_raw_transaction_info(&txid, None).unwrap();
    assert_eq!(tx.confirmations, None);

    // Mine blocks.
    rpc.generate_to_address(101, &address).unwrap();

    let tx = rpc.get_raw_transaction_info(&txid, None).unwrap();
    assert_eq!(tx.confirmations, Some(101));
}
