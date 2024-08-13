//! # Traits
//!
//! This crate implements [`jsonrpsee`] traits, using [`adapter`] functions.
//! This is the entry point for the RPC calls.

use super::adapter;
use crate::Client;
use bitcoin::BlockHash;
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

    #[method(name = "generatetoaddress")]
    async fn generatetoaddress(
        &self,
        nblocks: usize,
        address: String,
        maxtries: Option<usize>,
    ) -> Result<String, ErrorObjectOwned>;

    #[method(name = "getrawtransaction")]
    async fn getrawtransaction(
        &self,
        txid: String,
        verbose: Option<bool>,
        blockhash: Option<BlockHash>,
    ) -> Result<String, ErrorObjectOwned>;

    #[method(name = "sendrawtransaction")]
    async fn sendrawtransaction(
        &self,
        hexstring: String,
        maxfeerate: Option<usize>,
    ) -> Result<String, ErrorObjectOwned>;

    #[method(name = "getnewaddress")]
    async fn getnewaddress(
        &self,
        label: Option<String>,
        address_type: Option<String>,
    ) -> Result<String, ErrorObjectOwned>;

    #[method(name = "gettransaction")]
    async fn gettransaction(
        &self,
        txid: String,
        include_watchonly: Option<bool>,
        verbose: Option<bool>,
    ) -> Result<String, ErrorObjectOwned>;

    #[method(name = "sendtoaddress")]
    async fn sendtoaddress(
        &self,
        address: String,
        amount: String,
        comment: Option<&str>,
        comment_to: Option<&str>,
        subtractfeefromamount: Option<bool>,
        replaceable: Option<bool>,
        conf_target: Option<u32>,
        estimate_mode: Option<&str>,
        avoid_reuse: Option<bool>,
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

    async fn generatetoaddress(
        &self,
        nblocks: usize,
        address: String,
        maxtries: Option<usize>,
    ) -> Result<String, ErrorObjectOwned> {
        to_jsonrpsee_error(adapter::generatetoaddress(self, nblocks, address, maxtries))
    }

    async fn getrawtransaction(
        &self,
        txid: String,
        verbose: Option<bool>,
        blockhash: Option<BlockHash>,
    ) -> Result<String, ErrorObjectOwned> {
        to_jsonrpsee_error(adapter::getrawtransaction(self, txid, verbose, blockhash))
    }

    async fn sendrawtransaction(
        &self,
        hexstring: String,
        maxfeerate: Option<usize>,
    ) -> Result<String, ErrorObjectOwned> {
        to_jsonrpsee_error(adapter::sendrawtransaction(self, hexstring, maxfeerate))
    }

    async fn getnewaddress(
        &self,
        label: Option<String>,
        address_type: Option<String>,
    ) -> Result<String, ErrorObjectOwned> {
        to_jsonrpsee_error(adapter::getnewaddress(self, label, address_type))
    }

    async fn gettransaction(
        &self,
        txid: String,
        include_watchonly: Option<bool>,
        verbose: Option<bool>,
    ) -> Result<String, ErrorObjectOwned> {
        to_jsonrpsee_error(adapter::gettransaction(
            self,
            txid,
            include_watchonly,
            verbose,
        ))
    }

    async fn sendtoaddress(
        &self,
        address: String,
        amount: String,
        comment: Option<&str>,
        comment_to: Option<&str>,
        subtractfeefromamount: Option<bool>,
        replaceable: Option<bool>,
        conf_target: Option<u32>,
        estimate_mode: Option<&str>,
        avoid_reuse: Option<bool>,
    ) -> Result<String, ErrorObjectOwned> {
        to_jsonrpsee_error(adapter::sendtoaddress(
            self,
            address,
            amount,
            comment,
            comment_to,
            subtractfeefromamount,
            replaceable,
            conf_target,
            estimate_mode,
            avoid_reuse,
        ))
    }
}

/// Helper for converting ledger error to [`jsonrpsee`] error.
fn to_jsonrpsee_error<T>(input: Result<T, bitcoincore_rpc::Error>) -> Result<T, ErrorObjectOwned> {
    match input {
        Ok(res) => Ok(res),
        Err(e) => Err(ErrorObjectOwned::owned(0x45, e.to_string(), None::<String>)),
    }
}
