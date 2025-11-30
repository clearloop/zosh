//! Runtime library for the zosh bridge

use anyhow::Result;
use std::sync::Arc;
use tokio::sync::Mutex;
pub use {config::Config, hook::Hook, pool::Pool, storage::Storage};

mod author;
mod config;
mod hook;
mod import;
mod pool;
pub mod storage;

/// The runtime of the zosh bridge
pub struct Runtime<C: Config> {
    /// The hook for the runtime
    pub hook: C::Hook,

    /// The mempool
    pub pool: Arc<Mutex<Pool>>,

    /// The storage for the runtime
    pub storage: C::Storage,
}

impl<C: Config> Runtime<C> {
    /// Create a new runtime
    pub async fn new(hook: C::Hook, storage: C::Storage, threshold: usize) -> Result<Self> {
        Ok(Self {
            hook,
            pool: Arc::new(Mutex::new(Pool::new(threshold))),
            storage,
        })
    }
}
