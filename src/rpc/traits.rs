//! # Traits

use crate::Client;
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
}

#[async_trait]
impl RpcServer for InnerRpc {
    async fn sendrawtransaction(&self, tx: String) -> Result<String, ErrorObjectOwned> {
        Ok(self.client.send_raw_transaction(tx).unwrap().to_string())
    }
}
