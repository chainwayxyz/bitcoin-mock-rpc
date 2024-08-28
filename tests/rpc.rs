//! # RPC Interface Tests
//!
//! These tests aims to show RPC server can get up and handle requests in
//! different conditions, like single or multi-threaded environments. Therefore
//! correctness of the call results aren't necessarily important for these test.
//! It is the job of other tests.

use bitcoin::absolute::Height;
use bitcoin::consensus::encode::deserialize_hex;
use bitcoin::consensus::Decodable;
use bitcoin::transaction::Version;
use bitcoin::{Amount, OutPoint, Transaction, TxIn, TxOut};
use bitcoin_mock_rpc::rpc::spawn_rpc_server;
use bitcoincore_rpc::RpcApi;
use jsonrpsee::core::client::ClientT;
use jsonrpsee::{http_client::HttpClient, rpc_params};

mod common;

#[tokio::test]
async fn check_server_availability() {
    let server = spawn_rpc_server(None, None).unwrap();
    let url = format!("http://{}", server.0);
    println!("Server URL: {url}");

    let client = HttpClient::builder().build(url).unwrap();
    let params = rpc_params![];

    let response: String = client.request("getnewaddress", params).await.unwrap();
    println!("Server response: {:?}", response);
}

#[test]
fn create_connection() {
    let server = spawn_rpc_server(None, None).unwrap();
    let url = server.0.to_string();
    println!("Server started at {url}");

    let _should_not_panic =
        bitcoincore_rpc::Client::new(url.as_str(), bitcoincore_rpc::Auth::None).unwrap();
}

#[test]
fn address_related_rpc_calls() {
    let server = spawn_rpc_server(None, None).unwrap();
    let url = server.0.to_string();
    println!("Server started at {url}");

    let rpc = bitcoincore_rpc::Client::new(url.as_str(), bitcoincore_rpc::Auth::None).unwrap();

    let address = rpc.get_new_address(None, None).unwrap();
    println!("New address: {address:?}");
}

#[test]
fn block_related_rpc_calls() {
    let server = spawn_rpc_server(None, None).unwrap();
    let url = server.0.to_string();
    println!("Server started at {url}");

    let rpc = bitcoincore_rpc::Client::new(url.as_str(), bitcoincore_rpc::Auth::None).unwrap();

    // Generate some blocks.
    let address = rpc.get_new_address(None, None).unwrap();
    rpc.generate_to_address(101, &address.assume_checked())
        .unwrap();

    let height = rpc.get_block_count().unwrap();
    assert_eq!(height, 101);

    let tip_hash = rpc.get_best_block_hash().unwrap();

    let block = rpc.get_block(&tip_hash).unwrap();
    let header = rpc.get_block_header(&tip_hash).unwrap();
    assert_eq!(block.header, header);

    let txout = rpc
        .get_tx_out(&block.txdata.first().unwrap().compute_txid(), 0, None)
        .unwrap()
        .unwrap();
    assert_eq!(txout.confirmations, 1);
}

#[test]
fn transaction_related_rpc_calls() {
    let server = spawn_rpc_server(None, None).unwrap();
    let url = server.0.to_string();
    println!("Server started at {url}");

    let rpc = bitcoincore_rpc::Client::new(url.as_str(), bitcoincore_rpc::Auth::None).unwrap();

    let witness = common::create_witness();
    let address = common::create_address_from_witness(witness.0);

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

    let _tx = rpc.get_raw_transaction(&txid, None).unwrap();

    let txin = TxIn {
        previous_output: OutPoint { txid, vout: 0 },
        witness: witness.1,
        ..Default::default()
    };
    let txout = TxOut {
        value: Amount::from_sat(0x45),
        script_pubkey: address.script_pubkey(),
    };
    let tx = Transaction {
        input: vec![txin],
        output: vec![txout],
        version: Version::TWO,
        lock_time: bitcoin::absolute::LockTime::Blocks(Height::ZERO),
    };

    let new_txid = rpc.send_raw_transaction(&tx).unwrap();
    assert_ne!(txid, new_txid);
}

#[test]
fn fund_sign_raw_transaction() {
    let server = spawn_rpc_server(None, None).unwrap();
    let url = server.0.to_string();
    println!("Server started at {url}");

    let rpc = bitcoincore_rpc::Client::new(url.as_str(), bitcoincore_rpc::Auth::None).unwrap();

    let address = rpc.get_new_address(None, None).unwrap().assume_checked();

    let txout = TxOut {
        value: Amount::from_sat(0x45),
        script_pubkey: address.script_pubkey(),
    };
    let tx = Transaction {
        input: vec![],
        output: vec![txout],
        version: Version::TWO,
        lock_time: bitcoin::absolute::LockTime::Blocks(Height::ZERO),
    };

    if rpc.send_raw_transaction(&tx).is_ok() {
        assert!(false);
    }

    let new_tx = rpc.fund_raw_transaction(&tx, None, None).unwrap();
    assert_ne!(new_tx.change_position, -1);
    let new_tx = String::consensus_decode(&mut new_tx.hex.as_slice()).unwrap();
    let new_tx = deserialize_hex::<Transaction>(&new_tx).unwrap();
    assert_ne!(tx, new_tx);
}
