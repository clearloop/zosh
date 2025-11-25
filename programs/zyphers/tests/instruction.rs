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
        let mut mollusk = Mollusk::new(&zorch::ID, "zorch");

        // Add SPL Token program
        mollusk.add_program(
            &solana_sdk::pubkey!("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"),
            "spl_token",
            &solana_sdk::bpf_loader_upgradeable::id(),
        );

        Self {
            mollusk,
            payer: Pubkey::new_unique(),
        }
    }

    /// Create a new account
    pub fn account() -> AccountSharedData {
        let mut account = AccountSharedData::default();
        account.set_lamports(10_000_000_000);
        account
    }

    /// Create a native program account (for system program, etc.)
    pub fn native_program_account() -> AccountSharedData {
        let mut account = AccountSharedData::default();
        account.set_executable(true);
        account.set_owner(solana_sdk::native_loader::id());
        account.set_lamports(1_000_000_000);
        account
    }

    /// Create a BPF program account (for token program, etc.)
    pub fn bpf_program_account() -> AccountSharedData {
        let mut account = AccountSharedData::default();
        account.set_executable(true);
        account.set_owner(solana_sdk::bpf_loader_upgradeable::id());
        account.set_lamports(1_000_000_000);
        account
    }
}
