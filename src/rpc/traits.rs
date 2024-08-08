//! # Traits

use jsonrpsee::core::async_trait;
use jsonrpsee::proc_macros::rpc;
use jsonrpsee::types::ErrorObjectOwned;

#[rpc(server)]
pub trait Rpc {
    #[method(name = "send_raw_transaction")]
    async fn send_raw_transaction(&self, tx: String) -> Result<String, ErrorObjectOwned>;
}

#[async_trait]
impl RpcServer for () {
    async fn send_raw_transaction(&self, tx: String) -> Result<String, ErrorObjectOwned> {
        Ok(format!("Received: {tx}"))
    }
}
