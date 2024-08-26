//! # RPC Server Starter

use bitcoin_mock_rpc::rpc::spawn_rpc_server;
use std::env;

fn main() {
    println!("Bitcoin Mock Rpc (C) Chainway, 2024");
    println!(
        "Usage: {} [HOST] [PORT]",
        env::args().collect::<Vec<String>>().first().unwrap()
    );

    let server_info = handle_args();
    let server = spawn_rpc_server(server_info.0.as_deref(), server_info.1).unwrap();
    println!("Server started at {}", server.0);

    server.1.join().unwrap()
}

fn handle_args() -> (Option<String>, Option<u16>) {
    let mut ret = (None, None);

    let args: Vec<String> = env::args().collect();

    if let Some(host) = args.get(1) {
        ret.0 = Some(host.to_owned());
    };

    if let Some(port) = args.get(2) {
        ret.1 = Some(port.parse::<u16>().unwrap());
    };

    ret
}
