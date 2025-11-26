//! Tests for the instructions

use mollusk_svm::Mollusk;
use solana_account::Account;
use solana_sdk::{
    account::{AccountSharedData, WritableAccount},
    instruction::Instruction,
    pubkey::Pubkey,
    signature::Keypair,
    signer::Signer,
};
use zorch::{api, types::MintEntry};

mod external;
mod internal;
mod threshold;

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

    /// Create an account owned by the zorch program
    pub fn program_owned_account() -> AccountSharedData {
        let mut account = AccountSharedData::default();
        account.set_owner(zorch::ID);
        account.set_lamports(10_000_000_000);
        account
    }

    /// Create a token account owned by the token program
    pub fn token_account() -> AccountSharedData {
        let mut account = AccountSharedData::default();
        account.set_owner(api::pda::TOKEN_PROGRAM);
        account.set_lamports(10_000_000_000);
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

    /// Build accounts for the metadata instruction
    pub fn metadata_accounts(&self, authority: Pubkey) -> Vec<(Pubkey, Account)> {
        vec![
            (authority, Test::account().into()),
            (
                api::pda::bridge_state(),
                Test::program_owned_account().into(),
            ),
            (api::pda::zec_mint(), Test::account().into()),
            (api::pda::metadata(), Default::default()),
            (
                api::pda::TOKEN_METADATA_PROGRAM,
                Test::bpf_program_account().into(),
            ),
            (
                api::pda::SYSTEM_PROGRAM,
                Test::native_program_account().into(),
            ),
            self.mollusk.sysvars.keyed_account_for_rent_sysvar(),
        ]
    }

    /// Build accounts for the mint instruction (threshold action)
    pub fn mint_accounts(
        &self,
        recipient_token_accounts: Vec<(Pubkey, Account)>,
    ) -> Vec<(Pubkey, Account)> {
        let mut accounts = vec![
            (self.payer, Test::account().into()),
            (api::pda::bridge_state(), Test::account().into()),
            (api::pda::zec_mint(), Test::account().into()),
            (api::pda::TOKEN_PROGRAM, Test::bpf_program_account().into()),
            (
                api::pda::SYSTEM_PROGRAM,
                Test::native_program_account().into(),
            ),
            (
                api::pda::INSTRUCTIONS_SYSVAR,
                Test::native_program_account().into(),
            ),
        ];

        // Add recipient token accounts as remaining accounts
        accounts.extend(recipient_token_accounts);

        accounts
    }

    /// Build accounts for the burn instruction
    pub fn burn_accounts(
        &self,
        signer: Pubkey,
        signer_token_account: Pubkey,
    ) -> Vec<(Pubkey, Account)> {
        vec![
            (signer, Test::account().into()),
            (signer_token_account, Test::token_account().into()),
            (api::pda::zec_mint(), Test::account().into()),
            (
                api::pda::bridge_state(),
                Test::program_owned_account().into(),
            ),
            (api::pda::TOKEN_PROGRAM, Test::bpf_program_account().into()),
        ]
    }

    /// Build accounts for the validators instruction (threshold action)
    pub fn validators_accounts(&self) -> Vec<(Pubkey, Account)> {
        vec![
            (self.payer, Test::account().into()),
            (
                api::pda::bridge_state(),
                Test::program_owned_account().into(),
            ),
            (
                api::pda::SYSTEM_PROGRAM,
                Test::native_program_account().into(),
            ),
            (
                api::pda::INSTRUCTIONS_SYSVAR,
                Test::native_program_account().into(),
            ),
        ]
    }

    /// Sign the mint instruction
    pub fn mint_signatures(&self, nonce: u64, mints: Vec<MintEntry>) -> ([u8; 64], Instruction) {
        let mut message = nonce.to_le_bytes().to_vec();
        for mint in &mints {
            message.extend_from_slice(mint.recipient.as_ref());
            message.extend_from_slice(&mint.amount.to_le_bytes());
        }

        let signature = self.pair.sign_message(&message);
        let instruction = solana_ed25519_program::new_ed25519_instruction_with_signature(
            &message,
            &signature.as_array(),
            self.payer.as_array(),
        );
        (*signature.as_array(), instruction)
    }

    /// Sign the mint instruction
    pub fn validators_signatures(
        &self,
        nonce: u64,
        validators: Vec<Pubkey>,
        threshold: u8,
    ) -> ([u8; 64], Instruction) {
        let mut message = nonce.to_le_bytes().to_vec();
        message.extend_from_slice(&threshold.to_le_bytes());
        for validator in &validators {
            message.extend_from_slice(validator.as_ref());
        }

        let signature = self.pair.sign_message(&message);
        let instruction = solana_ed25519_program::new_ed25519_instruction_with_signature(
            &message,
            &signature.as_array(),
            self.payer.as_array(),
        );
        (*signature.as_array(), instruction)
    }
}

/// Generate a vector of unique pubkeys
pub fn pubkeys(count: u8) -> Vec<Pubkey> {
    (0..count)
        .map(|i| Pubkey::new_from_array([i; 32]))
        .collect::<Vec<_>>()
}
