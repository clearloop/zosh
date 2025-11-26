//! Solana client library for the Zorch bridge program
//!
//! This crate provides a high-level client built on `anchor-client` for interacting
//! with the Zorch program on Solana. It includes:
//! - Type-safe transaction building via Anchor
//! - Ed25519 signature helpers for threshold actions
//! - PDA derivation helpers
//! - State reading utilities

pub mod client;

// Re-export commonly used types
pub use client::ZorchClient;

// Re-export Zorch program types for convenience
pub use zorch::{types::MintEntry, BridgeState, BurnEvent, MintEvent, ValidatorSetUpdated};

// Re-export anchor-client for advanced usage
pub use anchor_client;
