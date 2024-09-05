//! # RPC Interface
//!
//! This crate provides an RPC server that will act like the real Bitcoin RPC
//! interface.

use crate::{Client, RpcApiWrapper};
use jsonrpsee::server::middleware::rpc::RpcServiceT;
use jsonrpsee::server::{RpcServiceBuilder, Server};
use jsonrpsee::types::Request;
use std::thread::JoinHandle;
use std::{io::Error, net::SocketAddr, net::TcpListener};
use traits::RpcServer;

mod adapter;
#[allow(clippy::too_many_arguments)]
mod traits;

/// Logger middleware.
#[derive(Clone)]
pub struct Logger<S>(S);
impl<'a, S> jsonrpsee::server::middleware::rpc::RpcServiceT<'a> for Logger<S>
where
    S: RpcServiceT<'a> + Send + Sync,
{
    type Future = S::Future;

    /// This will get called for every RPC request.
    fn call(&self, req: Request<'a>) -> Self::Future {
        tracing::info!(
            "Received RPC call: {}, with parameters: {:?}",
            req.method,
            req.params
        );

        self.0.call(req)
    }
}

/// Spawns an RPC server for the mock blockchain.
///
/// # Parameters
///
/// - host: Optional host. If is `None`, `127.0.0.1` will be used
/// - port: Optional port. If is `None`, a random port (assigned by OS) for
/// `host` will be used
///
/// # Returns
///
/// - `SocketAddr`: Address of the server
/// - `JoinHandle`: Server's handle that **must not be dropped** as long as
/// server lives
#[tracing::instrument]
pub fn spawn_rpc_server(
    host: Option<&str>,
    port: Option<u16>,
) -> Result<(SocketAddr, JoinHandle<()>), Error> {
    let host = host.unwrap_or("127.0.0.1");
    let url = match port {
        Some(p) => format!("{}:{}", host, p),
        None => TcpListener::bind((host, 0))?.local_addr()?.to_string(),
    };

    tracing::trace!("Starting a new RPC server at {url}");

    Ok(start_server_thread(url))
}

/// Starts a thread that hosts RPC server.
///
/// # Parameters
///
/// - url: Server's intended address
///
/// # Returns
///
/// - `SocketAddr`: Address of the server
/// - `JoinHandle`: Server's handle that must live as long as server
pub fn start_server_thread(url: String) -> (SocketAddr, JoinHandle<()>) {
    let (tx, rx) = std::sync::mpsc::channel();

    let handle = std::thread::spawn(move || {
        let mut rt = tokio::runtime::Builder::new_multi_thread();
        rt.enable_all();
        let rt = rt.build().unwrap();
        tracing::trace!("New Tokio runtime is created for server with URL {url}");

        rt.block_on(async {
            let rpc_middleware = RpcServiceBuilder::new().layer_fn(Logger);

            let server = Server::builder()
                .set_rpc_middleware(rpc_middleware)
                .build(url.clone())
                .await
                .unwrap();

            let address = server.local_addr().unwrap();

            // Start server.
            let client = Client::new(&url, bitcoincore_rpc::Auth::None).unwrap();
            let handle = server.start(client.into_rpc());

            // Server is up and we can notify that it is.
            tx.send(address).expect("Could not send socket address.");

            // Run forever.
            handle.stopped().await
        });
    });

    let address = rx
        .recv()
        .expect("Could not receive socket address from channel.");

    tracing::trace!("Server started for URL {address:?}");

    (address, handle)
}

#[cfg(test)]
mod tests {
    #[test]
    fn spawn_rpc_server() {
        let server = super::spawn_rpc_server(None, None).unwrap();
        println!("Server started at {}", server.0);
    }
}
