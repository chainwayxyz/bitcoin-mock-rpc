//! # RPC Server

use super::{traits::RpcServer, InnerRpc};
use crate::{ledger::errors::LedgerError, Client, RpcApiWrapper};
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
    let rpc = InnerRpc {
        client: Client::new(url, bitcoincore_rpc::Auth::None).unwrap(),
    };
    let handle = server.start(rpc.into_rpc());

    // Run server, till' it's shut down manually.
    tokio::spawn(handle.clone().stopped());

    Ok((addr, handle))
}
