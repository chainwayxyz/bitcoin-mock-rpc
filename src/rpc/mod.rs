//! # RPC Interface
//!
//! This crate provides an RPC server that will act like the real Bitcoin RPC
//! interface.

use crate::ledger::errors::LedgerError;
use jsonrpsee::server::ServerHandle;
use server::run_server;
use std::{io::Error, net::SocketAddr, net::TcpListener};
use traits::InnerRpc;

mod server;
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
    let host = match host {
        Some(h) => h,
        None => "127.0.0.1",
    };
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

/// Finds the first empty port for the given `host`.
fn find_empty_port(host: &str) -> Result<u16, Error> {
    for port in 1..0xFFFFu16 {
        if let Ok(_) = TcpListener::bind((host, port)) {
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
