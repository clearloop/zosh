//! Zosh JSON RPC API.

use jsonrpsee::{core::SubscriptionResult, proc_macros::rpc, types::ErrorObjectOwned};
use serde::{Deserialize, Serialize};
use zcore::{Block, State};

pub mod server;

#[cfg_attr(
    all(feature = "client", feature = "server"),
    rpc(client, server, namespace = "zosh")
)]
#[cfg_attr(feature = "server", rpc(server))]
#[cfg_attr(feature = "client", rpc(client))]
pub trait Api {
    /// Async method call example.
    #[method(name = "chainInfo")]
    async fn chain(&self) -> Result<State, ErrorObjectOwned>;

    /// Subscribe to new blocks.
    #[subscription(name = "subscribeBlock", item = BlockInterface)]
    async fn subscribe_block(&self) -> SubscriptionResult;

    /// Subscribe to transactions status.
    #[subscription(name = "subscribeTransaction", item = Vec<u8>)]
    async fn subscribe_transaction(&self, txid: Vec<u8>) -> SubscriptionResult;
}

/// Block interface for the UI
#[derive(Debug, Serialize, Deserialize)]
pub struct BlockInterface {
    pub block: Vec<u8>,
}

impl BlockInterface {
    /// Convert the block interface into a block
    pub fn into_block(&self) -> anyhow::Result<Block> {
        Ok(postcard::from_bytes(&self.block)?)
    }
}
