//! The development node services

use crate::dev::Development;
use anyhow::Result;
use runtime::{Pool, Runtime};
use std::sync::Arc;
use sync::Event;
use tokio::sync::{mpsc, Mutex};

/// Start the relay service
pub async fn relay(_pool: Arc<Mutex<Pool>>, _rx: mpsc::Receiver<Event>) -> Result<()> {
    Ok(())
}

/// Start the authoring service
pub async fn authoring(_runtime: Runtime<Development>) -> Result<()> {
    Ok(())
}
