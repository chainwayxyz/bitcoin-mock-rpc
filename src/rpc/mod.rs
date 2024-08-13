//! # RPC Interface
//!
//! This crate provides an RPC server that will act like the real Bitcoin RPC
//! interface.

use crate::ledger::errors::LedgerError;
use crate::{Client, RpcApiWrapper};
use jsonrpsee::server::Server;
use jsonrpsee::server::ServerHandle;
use std::{io::Error, net::SocketAddr, net::TcpListener};
use traits::RpcServer;

mod adapter;
#[allow(clippy::too_many_arguments)]
mod traits;

pub struct MockRpc {
    pub socket_address: SocketAddr,
    pub handle: ServerHandle,
}

/// Spawns an RPC server for the mock blockchain.
///
/// # Parameters
///
/// - host: Optional host. If is `None`, `127.0.0.1` will be used
/// - port: Optional port. If is `None`, first available port for `host` will be used
///
/// # Returns
///
/// URL on success, `std::io::Error` otherwise.
pub async fn spawn_rpc_server(host: Option<&str>, port: Option<u16>) -> Result<MockRpc, Error> {
    let host = host.unwrap_or("127.0.0.1");
    let port = match port {
        Some(p) => p,
        None => find_empty_port(host)?,
    };
    let url = format!("{}:{}", host, port);

    let (socket_address, handle) = run_server(url.as_str()).await.unwrap();

    Ok(MockRpc {
        socket_address,
        handle,
    })
}

pub async fn run_server(url: &str) -> Result<(SocketAddr, ServerHandle), LedgerError> {
    let server = match Server::builder().build(url).await {
        Ok(s) => s,
        Err(e) => return Err(LedgerError::Rpc(e.to_string())),
    };

    let addr = match server.local_addr() {
        Ok(a) => a,
        Err(e) => return Err(LedgerError::Rpc(e.to_string())),
    };

    let client = Client::new(url, bitcoincore_rpc::Auth::None).unwrap();
    let handle = server.start(client.into_rpc());

    // Run server, till' it's shut down manually.
    tokio::spawn(handle.clone().stopped());

    Ok((addr, handle))
}

/// Finds the first empty port for the given `host`.
fn find_empty_port(host: &str) -> Result<u16, Error> {
    for port in 1..0xFFFFu16 {
        if TcpListener::bind((host, port)).is_ok() {
            return Ok(port);
        }
    }

    Err(Error::other(LedgerError::Rpc(format!(
        "No port is available for host {}",
        host
    ))))
}

#[cfg(test)]
mod tests {
    #[test]
    fn find_empty_port() {
        let host = "127.0.0.1";

        println!(
            "Port {} is empty for {}",
            super::find_empty_port(host).unwrap(),
            host
        );
    }

    #[tokio::test]
    async fn spawn_rpc_server() {
        let server = super::spawn_rpc_server(None, None).await.unwrap();
        println!("Server started at {}", server.socket_address);
    }
}
