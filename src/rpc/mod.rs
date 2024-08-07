//! # RPC Interface
//!
//! This crate provides an RPC server that will act like the real Bitcoin RPC
//! interface.

use crate::ledger::errors::LedgerError;
use jsonrpsee::{server::Server, RpcModule};
use std::net::SocketAddr;
use std::{io::Error, net::TcpListener};

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
pub async fn spawn_rpc_server(host: Option<&str>, port: Option<u16>) -> Result<String, Error> {
    let host = match host {
        Some(h) => h,
        None => "127.0.0.1",
    };
    let port = match port {
        Some(p) => p,
        None => find_empty_port(host)?,
    };
    let url = format!("{}:{}", host, port);

    let server_addr = {
        let url = url.as_str().parse::<SocketAddr>().unwrap();
        let server = Server::builder().build(url).await?;
        let mut module = RpcModule::new(());

        module.register_method("say_hello", |_, _, _| "lo").unwrap();

        let addr = server.local_addr()?;
        let handle = server.start(module);

        // In this example we don't care about doing shutdown so let's it run forever.
        // You may use the `ServerHandle` to shut it down or manage it yourself.
        tokio::spawn(handle.stopped());

        Result::<std::net::SocketAddr, Error>::Ok(addr)
    }?;

    Ok(format!("http://{}", server_addr))
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
        println!(
            "Server started at {}",
            super::spawn_rpc_server(None, None).await.unwrap()
        );
    }
}
