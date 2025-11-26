//! client library for the zorch program
#![cfg(not(target_os = "solana"))]

pub use instruction::*;

pub mod instruction;
pub mod pda;
