//! Transaction related integration tests.

use bitcoin::{Amount, OutPoint, Transaction, TxIn, TxOut};
use bitcoin_mock_rpc::{Client, RpcApiWrapper};
use bitcoincore_rpc::{Auth, RpcApi};
use std::thread;
use tokio::join;

mod common;

async fn send_raw_transaction_async(rpc: Client, tx: Transaction) {
    rpc.send_raw_transaction(&tx).unwrap();
}

#[test]
fn send_to_address_multi_threaded() {
    let rpc = Client::new("", Auth::None).unwrap();
    let cloned_rpc = rpc.clone();
    let address = rpc.get_new_address(None, None).unwrap().assume_checked();

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

        assert_eq!(
            cloned_rpc.get_balance(None, None).unwrap(),
            Amount::from_sat(0x45)
        );
    })
    .join()
    .unwrap();

    // Change made in other rpc connection should also be available here.
    assert_eq!(rpc.get_balance(None, None).unwrap(), Amount::from_sat(0x45));
}

#[test]
fn use_utxo_from_send_to_address() {
    let rpc = Client::new("", Auth::None).unwrap();

    let address = rpc.get_new_address(None, None).unwrap().assume_checked();
    let deposit_address = common::create_address_from_witness();

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
    let txout = common::create_txout(0x45, Some(deposit_address.script_pubkey()));
    let tx = common::create_transaction(vec![txin.clone()], vec![txout]);
    rpc.send_raw_transaction(&tx).unwrap();

    // Invalid tx.
    let txout = common::create_txout(0x45 * 0x45, Some(deposit_address.script_pubkey()));
    let tx = common::create_transaction(vec![txin], vec![txout]);
    if let Ok(_) = rpc.send_raw_transaction(&tx) {
        assert!(false);
    };
}

#[tokio::test]
async fn send_get_raw_transaction_async() {
    let rpc = Client::new("", Auth::None).unwrap();

    let address = rpc.get_new_address(None, None).unwrap().assume_checked();
    let deposit_address = common::create_address_from_witness();

    // Create some funds to user.
    let txid1 = rpc
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
    let txid2 = rpc
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
    assert_eq!(
        rpc.get_balance(None, None).unwrap(),
        Amount::from_sat(0x45 * 0x45 * 2)
    );

    let txin1 = TxIn {
        previous_output: OutPoint {
            txid: txid1,
            vout: 0,
        },
        ..Default::default()
    };
    let txout = TxOut {
        value: Amount::from_sat(0x45),
        script_pubkey: address.script_pubkey(),
    };
    let tx1 = common::create_transaction(vec![txin1.clone()], vec![txout]);

    let txin2 = TxIn {
        previous_output: OutPoint {
            txid: txid2,
            vout: 0,
        },
        ..Default::default()
    };
    let txout = TxOut {
        value: Amount::from_sat(0x1F),
        script_pubkey: address.script_pubkey(),
    };
    let tx2 = common::create_transaction(vec![txin2.clone()], vec![txout]);

    let async_thr1 = send_raw_transaction_async(rpc.clone(), tx1.clone());
    let async_thr2 = send_raw_transaction_async(rpc.clone(), tx2.clone());

    join!(async_thr1, async_thr2);

    // We burned our money. We should only have the amount we send it to ourselves.
    assert_eq!(
        rpc.get_balance(None, None).unwrap(),
        Amount::from_sat(0x1F + 0x45)
    );

    // Send some funds to some other user.
    let txin = common::create_txin(tx1.compute_txid());
    let txout = TxOut {
        value: Amount::from_sat(0x45),
        script_pubkey: deposit_address.script_pubkey(),
    };
    let tx1 = common::create_transaction(vec![txin], vec![txout]);

    let txin = common::create_txin(tx2.compute_txid());
    let txout = TxOut {
        value: Amount::from_sat(0x1F),
        script_pubkey: deposit_address.script_pubkey(),
    };
    let tx2 = common::create_transaction(vec![txin], vec![txout]);

    let async_thr1 = send_raw_transaction_async(rpc.clone(), tx1);
    let async_thr2 = send_raw_transaction_async(rpc.clone(), tx2);

    join!(async_thr1, async_thr2);

    // Balance should be lower now.
    assert_eq!(rpc.get_balance(None, None).unwrap(), Amount::from_sat(0));
}
