//! client library for the zorch program
#![cfg(not(target_os = "solana"))]

use anchor_client::{
    solana_sdk::{commitment_config::CommitmentConfig, pubkey::Pubkey, signature::Keypair},
    Client, Cluster, Program,
};
use anyhow::Result;
pub use instruction::*;
use mpl_token_metadata::accounts::Metadata;
use solana_sdk::signature::Signature;
use std::rc::Rc;

mod config;
pub mod instruction;
pub mod pda;
mod signatures;

/// Main client for interacting with the Zorch program
pub struct ZorchClient {
    /// Anchor client program instance
    program: Program<Rc<Keypair>>,
}

impl ZorchClient {
    /// Create a new ZorchClient
    pub fn new(cluster_url: String, ws_url: String, payer: Keypair) -> Result<Self> {
        let client = Client::new_with_options(
            Cluster::Custom(cluster_url, ws_url),
            Rc::new(payer),
            CommitmentConfig::confirmed(),
        );

        let program = client.program(crate::ID)?;
        Ok(Self { program })
    }

    /// Get the payer's public key
    pub fn payer(&self) -> Pubkey {
        self.program.payer()
    }

    /// Get the program client
    pub fn program(&self) -> &Program<Rc<Keypair>> {
        &self.program
    }

    /// Read the current bridge state
    pub async fn bridge_state(&self) -> Result<crate::state::BridgeState> {
        let bridge_state_pubkey = pda::bridge_state();
        let bridge_state: crate::state::BridgeState =
            self.program.account(bridge_state_pubkey).await?;
        Ok(bridge_state)
    }

    /// Read the current metadata
    pub async fn metadata(&self) -> Result<Metadata> {
        let metadata_pubkey = pda::metadata();
        let account_data = self
            .program
            .rpc()
            .get_account_data(&metadata_pubkey)
            .await?;

        Metadata::from_bytes(&account_data).map_err(Into::into)
    }

    /// Initialize the bridge with initial validator set
    pub async fn initialize(&self, validators: Vec<Pubkey>, threshold: u8) -> Result<Signature> {
        let bridge_state = pda::bridge_state();
        let zec_mint = pda::zec_mint();
        let tx = self
            .program
            .request()
            .accounts(crate::accounts::Initialize {
                payer: self.program.payer(),
                bridge_state,
                zec_mint,
                system_program: pda::SYSTEM_PROGRAM,
                token_program: pda::TOKEN_PROGRAM,
                rent: pda::RENT,
            })
            .args(crate::instruction::Initialize {
                validators,
                threshold,
            })
            .send()
            .await?;
        Ok(tx)
    }

    /// Update token metadata (authority only)
    pub async fn update_metadata(&self, name: String, symbol: String, uri: String) -> Result<()> {
        let bridge_state = pda::bridge_state();
        let zec_mint = pda::zec_mint();
        let metadata = pda::metadata();

        let _tx = self
            .program
            .request()
            .accounts(crate::accounts::UpdateMetadata {
                authority: self.program.payer(),
                bridge_state,
                zec_mint,
                metadata,
                token_metadata_program: pda::TOKEN_METADATA_PROGRAM,
                system_program: pda::SYSTEM_PROGRAM,
                sysvar_instructions: pda::INSTRUCTIONS_SYSVAR,
            })
            .args(crate::instruction::Metadata { name, symbol, uri })
            .send()
            .await?;

        Ok(())
    }

    /// Mint sZEC to recipients (threshold action)
    pub async fn send_mint(
        &self,
        mint_entries: Vec<crate::types::MintEntry>,
        signatures: Vec<[u8; 64]>,
    ) -> Result<()> {
        anyhow::ensure!(!mint_entries.is_empty(), "No mint entries provided");
        let bridge_state = pda::bridge_state();
        let zec_mint = pda::zec_mint();

        // Get recipient token accounts
        let mut remaining_accounts = Vec::new();
        for entry in &mint_entries {
            let token_account = spl_associated_token_account::get_associated_token_address(
                &entry.recipient,
                &zec_mint,
            );
            remaining_accounts.push(anchor_client::solana_sdk::instruction::AccountMeta::new(
                token_account,
                false,
            ));
        }

        let _tx = self
            .program
            .request()
            .accounts(crate::accounts::MintZec {
                payer: self.program.payer(),
                bridge_state,
                zec_mint,
                token_program: pda::TOKEN_PROGRAM,
                system_program: pda::SYSTEM_PROGRAM,
                instructions: pda::INSTRUCTIONS_SYSVAR,
            })
            .args(crate::instruction::Mint {
                mints: mint_entries,
                signatures,
            })
            .accounts(remaining_accounts)
            .send()
            .await?;

        Ok(())
    }

    /// Burn sZEC to bridge back to Zcash (public action)
    pub async fn send_burn(&self, amount: u64, zec_recipient: String) -> Result<()> {
        anyhow::ensure!(amount > 0, "Amount must be greater than 0");
        let bridge_state = pda::bridge_state();
        let zec_mint = pda::zec_mint();
        let signer_token_account = spl_associated_token_account::get_associated_token_address(
            &self.program.payer(),
            &zec_mint,
        );

        let _tx = self
            .program
            .request()
            .accounts(crate::accounts::BurnZec {
                signer: self.program.payer(),
                signer_token_account,
                zec_mint,
                bridge_state,
                token_program: pda::TOKEN_PROGRAM,
            })
            .args(crate::instruction::Burn {
                amount,
                zec_recipient,
            })
            .send()
            .await?;

        Ok(())
    }

    /// Update the validator set (threshold action)
    pub async fn update_validators(
        &self,
        new_validators: Vec<Pubkey>,
        new_threshold: u8,
        signatures: Vec<[u8; 64]>,
    ) -> Result<()> {
        anyhow::ensure!(!new_validators.is_empty(), "No validators provided");
        anyhow::ensure!(
            new_threshold > 0 && new_threshold as usize <= new_validators.len(),
            "Invalid threshold"
        );

        let bridge_state = pda::bridge_state();
        let _tx = self
            .program
            .request()
            .accounts(crate::accounts::Validators {
                payer: self.program.payer(),
                bridge_state,
                system_program: pda::SYSTEM_PROGRAM,
                instructions: pda::INSTRUCTIONS_SYSVAR,
            })
            .args(crate::instruction::Validators {
                new_validators,
                new_threshold,
                signatures,
            })
            .send()
            .await?;

        Ok(())
    }
}
