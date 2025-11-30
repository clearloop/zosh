//! The development node services

use anyhow::Result;
use runtime::Pool;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use zcore::ex::Bridge;

/// Start the relay service
pub async fn start(pool: Arc<Mutex<Pool>>, mut rx: mpsc::Receiver<Bridge>) -> Result<()> {
    Ok(())
}
