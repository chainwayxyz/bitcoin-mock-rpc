//! # Traits

use crate::Client;
use bitcoin::consensus::Encodable;
use bitcoin::hex::DisplayHex;
use bitcoin::{BlockHash, Txid};
use bitcoincore_rpc::RpcApi;
use jsonrpsee::core::async_trait;
use jsonrpsee::proc_macros::rpc;
use jsonrpsee::types::ErrorObjectOwned;

/// Holds ledger connection.
pub struct InnerRpc {
    pub client: Client,
}

#[rpc(server)]
pub trait Rpc {
    #[method(name = "sendrawtransaction")]
    async fn sendrawtransaction(&self, tx: String) -> Result<String, ErrorObjectOwned>;

    #[method(name = "getrawtransaction")]
    async fn getrawtransaction(
        &self,
        txid: Txid,
        block_hash: Option<BlockHash>,
    ) -> Result<String, ErrorObjectOwned>;
}

#[async_trait]
impl RpcServer for InnerRpc {
    async fn sendrawtransaction(&self, tx: String) -> Result<String, ErrorObjectOwned> {
        if let Ok(res) = self.client.send_raw_transaction(tx) {
            return Ok(res.to_string());
        };

        Err(ErrorObjectOwned::from(
            jsonrpsee::types::ErrorCode::InvalidParams,
        ))
    }

    async fn getrawtransaction(
        &self,
        txid: Txid,
        block_hash: Option<BlockHash>,
    ) -> Result<String, ErrorObjectOwned> {
        if let Ok(res) = self.client.get_raw_transaction(&txid, block_hash.as_ref()) {
            let mut hex: Vec<u8> = Vec::new();
            if let Err(_) = res.consensus_encode(&mut hex) {
                return Err(ErrorObjectOwned::from(
                    jsonrpsee::types::ErrorCode::InvalidParams,
                ));
            };

            return Ok(hex.to_hex_string(bitcoin::hex::Case::Upper));
        };

        Err(ErrorObjectOwned::from(
            jsonrpsee::types::ErrorCode::InvalidParams,
        ))
    }
}
