//! Tests for the instructions

use mollusk_svm::Mollusk;
use solana_account::Account;
use solana_sdk::{
    account::{AccountSharedData, WritableAccount},
    pubkey::Pubkey,
    signature::Keypair,
    signer::Signer,
};
mod internal;
use zorch::api;

/// Testing client for the instructions
pub struct Test {
    /// Mollusk VM client
    pub mollusk: Mollusk,

    /// Signer keypair
    pub payer: Pubkey,

    /// Signer keypair
    pub pair: Keypair,
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

        let pair = Keypair::new();
        Self {
            mollusk,
            payer: pair.pubkey(),
            pair,
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

    /// Initialize accounts for the initialize instruction
    pub fn initialize_accounts(&self) -> Vec<(Pubkey, Account)> {
        vec![
            (self.payer, Test::account().into()),
            (api::pda::bridge_state(), Default::default()),
            (api::pda::zec_mint(), Default::default()),
            (
                api::pda::SYSTEM_PROGRAM,
                Test::native_program_account().into(),
            ),
            (api::pda::TOKEN_PROGRAM, Test::bpf_program_account().into()),
            self.mollusk.sysvars.keyed_account_for_rent_sysvar(),
        ]
    }
}

/// Generate a vector of unique pubkeys
pub fn pubkeys(count: u8) -> Vec<Pubkey> {
    (0..count)
        .map(|i| Pubkey::new_from_array([i; 32]))
        .collect::<Vec<_>>()
}
