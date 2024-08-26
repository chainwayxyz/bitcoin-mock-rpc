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

    let client = bitcoincore_rpc::Client::new(url.as_str(), bitcoincore_rpc::Auth::None).unwrap();

    let _address = client.get_new_address(None, None).unwrap();

    // client
    //     .generate_to_address(101, &address.assume_checked())
    //     .unwrap();
}
