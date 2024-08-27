//! # RPC Interface Tests
//!
//! These tests aims to show RPC server can get up and handle requests in
//! different conditions, like single or multi-threaded environments. Therefore
//! correctness of the call results aren't necessarily important for these test.
//! It is the job of other tests.

use bitcoin_mock_rpc::rpc::spawn_rpc_server;
use bitcoincore_rpc::RpcApi;
use jsonrpsee::core::client::ClientT;
use jsonrpsee::{http_client::HttpClient, rpc_params};

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
fn address_related() {
    let server = spawn_rpc_server(None, None).unwrap();
    let url = server.0.to_string();
    println!("Server started at {url}");

    let rpc = bitcoincore_rpc::Client::new(url.as_str(), bitcoincore_rpc::Auth::None).unwrap();

    let address = rpc.get_new_address(None, None).unwrap();
    println!("New address: {address:?}");
}

#[test]
fn block_related() {
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
