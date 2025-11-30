//! Zosh JSON RPC API.

use jsonrpsee::{core::SubscriptionResult, proc_macros::rpc, types::ErrorObjectOwned};
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
    #[subscription(name = "subscribeBlock", item = Block)]
    async fn subscribe_block(&self) -> SubscriptionResult;

    /// Subscribe to transactions status.
    #[subscription(name = "subscribeTransaction", item = Vec<u8>)]
    async fn subscribe_transaction(&self, txid: Vec<u8>) -> SubscriptionResult;
}
