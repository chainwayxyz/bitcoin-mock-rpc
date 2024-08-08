//! # RPC Server

use super::traits::RpcServer;
use crate::ledger::errors::LedgerError;
use jsonrpsee::server::{Server, ServerHandle};
use std::net::SocketAddr;

pub async fn run_server(url: &str) -> Result<(SocketAddr, ServerHandle), LedgerError> {
    let server = match Server::builder().build(url).await {
        Ok(s) => s,
        Err(e) => return Err(LedgerError::Rpc(e.to_string())),
    };

    let addr = match server.local_addr() {
        Ok(a) => a,
        Err(e) => return Err(LedgerError::Rpc(e.to_string())),
    };
    let handle = server.start(().into_rpc());

    // Run server, till' it's shut down manually.
    tokio::spawn(handle.clone().stopped());

    Ok((addr, handle))
}
