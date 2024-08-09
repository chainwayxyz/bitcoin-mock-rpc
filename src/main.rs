//! # RPC Server Starter

use bitcoin_mock_rpc::rpc::spawn_rpc_server;

#[tokio::main]
async fn main() {
    println!("Bitcoin Mock Rpc (C) Chainway, 2024");

    let server = spawn_rpc_server(None, None).await.unwrap();
    println!("Server started at {}", server.socket_address);

    loop {
        if server.handle.is_stopped() {
            break;
        }
    }
}
