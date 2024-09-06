//! Integration tests for `raw_transaction` calls.

use bitcoin::{hashes::Hash, Amount, OutPoint, Transaction, TxIn, TxOut, Txid};
use bitcoin_mock_rpc::{Client, RpcApiWrapper};
use bitcoincore_rpc::{Auth, RpcApi};
use common::send_raw_transaction_async;
use tokio::join;

mod common;

#[test]
fn send_get_raw_transaction_with_change() {
    let rpc = Client::new("send_get_raw_transaction_with_change", Auth::None).unwrap();

    let witness = common::create_witness();
    let address = common::create_address_from_witness(witness.0);
    let witness2 = common::create_witness();
    let deposit_address = common::create_address_from_witness(witness2.0);

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

    let txin = TxIn {
        previous_output: OutPoint { txid, vout: 0 },
        witness: witness.1,
        ..Default::default()
    };
    let txout0 = common::create_txout(Amount::from_sat(0x45), deposit_address.script_pubkey());
    let txout1 = common::create_txout(Amount::from_sat(0x45 * 0x44), address.script_pubkey());
    let tx = common::create_transaction(vec![txin], vec![txout0, txout1]);
    let txid = rpc.send_raw_transaction(&tx).unwrap();

    assert_eq!(rpc.get_raw_transaction(&txid, None).unwrap(), tx);
}

#[test]
fn send_get_raw_transaction_without_change() {
    let rpc = Client::new("send_get_raw_transaction_without_change", Auth::None).unwrap();

    let witness = common::create_witness();
    let address = common::create_address_from_witness(witness.0);
    let witness2 = common::create_witness();
    let deposit_address = common::create_address_from_witness(witness2.0);

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

    let txin = TxIn {
        previous_output: OutPoint { txid, vout: 0 },
        witness: witness.1,
        ..Default::default()
    };
    let txout = common::create_txout(Amount::from_sat(0x45), deposit_address.script_pubkey());
    let tx = common::create_transaction(vec![txin], vec![txout]);
    let txid = rpc.send_raw_transaction(&tx).unwrap();

    // Because we can't check balance rn, this test is meaningless.

    assert_eq!(rpc.get_raw_transaction(&txid, None).unwrap(), tx);
}

#[tokio::test]
async fn send_get_raw_transaction_async() {
    let rpc = Client::new("send_get_raw_transaction_async", Auth::None).unwrap();

    let witness = common::create_witness();
    let address = common::create_address_from_witness(witness.0);
    let witness2 = common::create_witness();
    let deposit_address = common::create_address_from_witness(witness2.0);

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

    let txin1 = TxIn {
        previous_output: OutPoint {
            txid: txid1,
            vout: 0,
        },
        witness: witness.1.clone(),
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
        witness: witness.1.clone(),
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
    // assert_eq!(
    //     rpc.get_balance(None, None).unwrap(),
    //     Amount::from_sat(0x1F + 0x45)
    // );

    // Send some funds to some other user.
    let txin = TxIn {
        previous_output: OutPoint {
            txid: tx1.compute_txid(),
            vout: 0,
        },
        witness: witness.1.clone(),
        ..Default::default()
    };
    let txout = TxOut {
        value: Amount::from_sat(0x45),
        script_pubkey: deposit_address.script_pubkey(),
    };
    let tx1 = common::create_transaction(vec![txin], vec![txout]);

    let txin = TxIn {
        previous_output: OutPoint {
            txid: tx2.compute_txid(),
            vout: 0,
        },
        witness: witness.1.clone(),
        ..Default::default()
    };
    let txout = TxOut {
        value: Amount::from_sat(0x1F),
        script_pubkey: deposit_address.script_pubkey(),
    };
    let tx2 = common::create_transaction(vec![txin], vec![txout]);

    let async_thr1 = send_raw_transaction_async(rpc.clone(), tx1);
    let async_thr2 = send_raw_transaction_async(rpc.clone(), tx2);

    join!(async_thr1, async_thr2);

    // Balance should be lower now.
    // assert_eq!(rpc.get_balance(None, None).unwrap(), Amount::from_sat(0));
}

#[test]
#[should_panic]
fn send_raw_transaction_invalid_input() {
    let rpc = Client::new("send_raw_transaction_invalid_input", Auth::None).unwrap();

    let address = rpc.get_new_address(None, None).unwrap().assume_checked();

    assert_eq!(rpc.get_balance(None, None).unwrap(), Amount::from_sat(0));

    let txin = common::create_txin(Txid::all_zeros(), 0);
    let txout = common::create_txout(Amount::from_sat(0x45), address.script_pubkey());
    let tx = common::create_transaction(vec![txin], vec![txout]);

    // Input is not valid, this should panic.
    rpc.send_raw_transaction(&tx).unwrap();

    // This should also panic. User don't have any funds.
    assert_eq!(rpc.get_balance(None, None).unwrap(), Amount::from_sat(0x45));
}

#[test]
#[should_panic]
fn send_raw_transaction_insufficient_funds() {
    let rpc = Client::new("send_raw_transaction_insufficient_funds", Auth::None).unwrap();

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
    assert_eq!(
        rpc.get_balance(None, None).unwrap(),
        Amount::from_sat(0x45 * 0x45)
    );

    let txin = TxIn {
        previous_output: OutPoint { txid, vout: 0 },
        witness: witness.1,
        ..Default::default()
    };
    let txout = common::create_txout(
        Amount::from_sat(0x45 * 0x45 * 0x1F),
        address.script_pubkey(),
    );
    let tx = common::create_transaction(vec![txin], vec![txout]);

    // Input is not enough for output, this should panic.
    rpc.send_raw_transaction(&tx).unwrap();

    // This should also panic. User don't have that funds.
    assert_eq!(
        rpc.get_balance(None, None).unwrap(),
        Amount::from_sat(0x45 * 0x45 * 0x1F)
    );
}

#[test]
fn fund_sign_raw_transaction_with_wallet() {
    let rpc = Client::new("fund_sign_raw_transaction_with_wallet", Auth::None).unwrap();

    let address = rpc.get_new_address(None, None).unwrap().assume_checked();

    let txout = TxOut {
        value: Amount::from_sat(0x45),
        script_pubkey: address.script_pubkey(),
    };
    let tx = common::create_transaction(vec![], vec![txout]);

    // Lower input funds should be a problem.
    assert!(rpc.send_raw_transaction(&tx).is_err());

    let res = rpc.fund_raw_transaction(&tx, None, None).unwrap();
    let tx = bitcoin::consensus::encode::deserialize::<Transaction>(&res.hex).unwrap();

    // Not signed inputs should be a problem.
    assert!(rpc.send_raw_transaction(&tx).is_err());

    let res = rpc
        .sign_raw_transaction_with_wallet(&tx, None, None)
        .unwrap();
    let tx = bitcoin::consensus::encode::deserialize::<Transaction>(&res.hex).unwrap();

    rpc.send_raw_transaction(&tx).unwrap();
}
