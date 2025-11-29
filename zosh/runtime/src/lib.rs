//! Runtime library for the zosh bridge

use sync::Sync;
pub use {config::Config, hook::Hook, pool::Pool, storage::Storage};

mod author;
mod config;
mod hook;
mod import;
mod pool;
mod storage;
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
