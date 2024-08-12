//! # Traits

use super::adapter;
use crate::Client;
use jsonrpsee::core::async_trait;
use jsonrpsee::proc_macros::rpc;
use jsonrpsee::types::ErrorObjectOwned;

#[rpc(server)]
pub trait Rpc {
    #[method(name = "getbestblockhash")]
    async fn getbestblockhash(&self) -> Result<String, ErrorObjectOwned>;

    #[method(name = "getblock")]
    async fn getblock(
        &self,
        blockhash: String,
        verbosity: Option<usize>,
    ) -> Result<String, ErrorObjectOwned>;

    #[method(name = "getblockcount")]
    async fn getblockcount(&self) -> Result<usize, ErrorObjectOwned>;

    #[method(name = "getblockhash")]
    async fn getblockhash(&self, height: usize) -> Result<String, ErrorObjectOwned>;

    #[method(name = "getblockheader")]
    async fn getblockheader(
        &self,
        blockhash: String,
        verbose: Option<bool>,
    ) -> Result<String, ErrorObjectOwned>;

    #[method(name = "gettxout")]
    async fn gettxout(
        &self,
        txid: String,
        n: u32,
        include_mempool: Option<bool>,
    ) -> Result<String, ErrorObjectOwned>;
}

#[async_trait]
impl RpcServer for Client {
    async fn getbestblockhash(&self) -> Result<String, ErrorObjectOwned> {
        to_jsonrpsee_error(adapter::getbestblockhash(self))
    }

    async fn getblock(
        &self,
        blockhash: String,
        verbosity: Option<usize>,
    ) -> Result<String, ErrorObjectOwned> {
        to_jsonrpsee_error(adapter::getblock(self, blockhash, verbosity))
    }

    async fn getblockcount(&self) -> Result<usize, ErrorObjectOwned> {
        to_jsonrpsee_error(adapter::getblockcount(self))
    }

    async fn getblockhash(&self, height: usize) -> Result<String, ErrorObjectOwned> {
        to_jsonrpsee_error(adapter::getblockhash(self, height))
    }

    async fn getblockheader(
        &self,
        blockhash: String,
        verbose: Option<bool>,
    ) -> Result<String, ErrorObjectOwned> {
        to_jsonrpsee_error(adapter::getblockheader(self, blockhash, verbose))
    }

    async fn gettxout(
        &self,
        txid: String,
        n: u32,
        include_mempool: Option<bool>,
    ) -> Result<String, ErrorObjectOwned> {
        to_jsonrpsee_error(adapter::gettxout(self, txid, n, include_mempool))
    }
}

/// Helper for converting ledger error to [`jsonrpsee`] error.
fn to_jsonrpsee_error<T>(input: Result<T, bitcoincore_rpc::Error>) -> Result<T, ErrorObjectOwned> {
    match input {
        Ok(res) => Ok(res),
        Err(_) => Err(ErrorObjectOwned::from(
            jsonrpsee::types::ErrorCode::InvalidParams,
        )),
    }
}
