//! RPC implementation for the zosh node

use anyhow::Result;
use async_trait::async_trait;
use rpc::{
    server::{
        middleware, ErrorObjectOwned, PendingSubscriptionSink, RpcServiceBuilder, Server,
        SubscriptionManager, SubscriptionResult,
    },
    ApiServer,
};
use runtime::Storage;
use std::{net::SocketAddr, sync::Arc};
use zcore::State;

/// The response type
pub type Response<T> = core::result::Result<T, ErrorObjectOwned>;

/// The RPC implementation
pub struct Rpc<S: Storage> {
    /// The storage
    pub storage: Arc<S>,

    /// the subscription manager
    pub manager: SubscriptionManager,
}

impl<S: Storage> Rpc<S> {
    /// Create a new RPC instance
    pub fn new(storage: Arc<S>, manager: SubscriptionManager) -> Self {
        Self { storage, manager }
    }

    /// Start the RPC server
    pub async fn start(self, addr: SocketAddr) -> Result<()> {
        let rpc_middleware = RpcServiceBuilder::new().layer_fn(middleware::Logger);
        let server = Server::builder()
            .set_rpc_middleware(rpc_middleware)
            .build(addr)
            .await?;

        // start the rpc server
        let addr = server.local_addr()?;
        tracing::info!("Listening RPC on {}", addr);
        server.start(self.into_rpc()).stopped().await;
        Ok(())
    }
}

#[async_trait]
impl<S: Storage> ApiServer for Rpc<S> {
    /// Get the chain info
    async fn chain(&self) -> Response<State> {
        Ok(self.storage.state())
    }

    /// Subscribe to new blocks
    async fn subscribe_block(&self, sink: PendingSubscriptionSink) -> SubscriptionResult {
        let sink = sink.accept().await?;
        self.manager.block_sub.lock().await.push(sink);
        Ok(())
    }

    /// Subscribe to a transaction status
    async fn subscribe_transaction(
        &self,
        sink: PendingSubscriptionSink,
        txid: Vec<u8>,
    ) -> SubscriptionResult {
        let sink = sink.accept().await?;
        self.manager.transaction_sub.lock().await.push((txid, sink));
        Ok(())
    }
}
