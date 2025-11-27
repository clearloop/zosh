//! The subscription of the solana client

use crate::{solana::SolanaClient, Event};
use anyhow::Result;
use solana_rpc_client_types::config::RpcTransactionLogsFilter;
use tokio::sync::mpsc;

impl SolanaClient {
    /// Subscribe to the solana client
    pub async fn subscribe(&self, tx: mpsc::Sender<Event>) -> Result<()> {
        RpcTransactionLogsFilter::Mentions(vec![zosh::ID.to_string()]);

        Ok(())
    }
}
