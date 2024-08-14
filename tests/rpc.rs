//! # RPC Interface Tests

use bitcoin_mock_rpc::rpc::spawn_rpc_server;
use bitcoincore_rpc::RpcApi;

#[tokio::test]
async fn create_connection() {
    let address = spawn_rpc_server(None, None).await.unwrap();
    let url = address.to_string();

    let _should_not_panic =
        bitcoincore_rpc::Client::new(url.as_str(), bitcoincore_rpc::Auth::None).unwrap();
    println!("Server started at {url}");
}

#[tokio::test]
#[ignore = "causes infinite loop"]
async fn address_related() {
    let address = spawn_rpc_server(None, None).await.unwrap();
    let url = address.to_string();

    let client = bitcoincore_rpc::Client::new(url.as_str(), bitcoincore_rpc::Auth::None).unwrap();
    println!("Server started at {url}");

    let address = client.get_new_address(None, None).unwrap();

    client
        .generate_to_address(101, &address.assume_checked())
        .unwrap();
}
