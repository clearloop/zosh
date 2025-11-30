//! Server implementation for the Spacejam JSON RPC API.

#![cfg(feature = "server")]

pub use jsonrpsee::{
    core::{middleware::RpcServiceBuilder, SubscriptionResult},
    server::Server,
    types::{ErrorCode, ErrorObjectOwned},
    PendingSubscriptionSink,
};
pub use sub::SubscriptionManager;

pub mod middleware;
mod sub;
