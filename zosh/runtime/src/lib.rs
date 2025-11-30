//! Runtime library for the zosh bridge

use anyhow::Result;
use sync::Sync;
pub use {config::Config, hook::Hook, pool::Pool, storage::Storage};

mod author;
mod config;
mod hook;
mod import;
mod pool;
pub mod storage;
mod validate;

/// The runtime of the zosh bridge
pub struct Runtime<C: Config> {
    /// The hook for the runtime
    pub hook: C::Hook,

    /// The mempool
    pub pool: Pool,

    /// The sync clients for verification usages
    pub sync: Sync,

    /// The storage for the runtime
    pub storage: C::Storage,
}

impl<C: Config> Runtime<C> {
    /// Create a new runtime
    pub async fn new(hook: C::Hook, storage: C::Storage) -> Result<Self> {
        let config = sync::Config::load()?;
        Ok(Self {
            hook,
            pool: Pool::default(),
            sync: Sync::new(&config).await?,
            storage,
        })
    }
}
