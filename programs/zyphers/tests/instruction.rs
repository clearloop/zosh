//! Tests for the instructions

use mollusk_svm::Mollusk;
use solana_sdk::{
    account::{AccountSharedData, WritableAccount},
    pubkey::Pubkey,
};

mod internal;

/// Generate a vector of unique pubkeys
pub fn pubkeys(count: u8) -> Vec<Pubkey> {
    (0..count)
        .map(|i| Pubkey::new_from_array([i; 32]))
        .collect::<Vec<_>>()
}

/// Testing client for the instructions
pub struct Test {
    /// Mollusk VM client
    pub mollusk: Mollusk,

    /// Signer keypair
    pub payer: Pubkey,
}

impl Test {
    /// Create a new Test instance
    pub fn new() -> Self {
        Self {
            mollusk: Mollusk::new(&zyphers::ID, "zyphers"),
            payer: Pubkey::new_unique(),
        }
    }

    /// Create a new account
    pub fn account() -> AccountSharedData {
        let mut account = AccountSharedData::default();
        account.set_lamports(10_000_000_000);
        account
    }
}
