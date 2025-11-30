//! Server implementation for the Spacejam JSON RPC API.

#![cfg(feature = "server")]

pub use sub::SubscriptionManager;

pub mod middleware;
mod sub;
