//! client library for the zosh program
#![cfg(not(target_os = "solana"))]

use anchor_client::{
    solana_sdk::{commitment_config::CommitmentConfig, pubkey::Pubkey, signature::Keypair},
    Client, Cluster, Program,
};
use anchor_lang::AccountDeserialize;
use anchor_spl::token::TokenAccount;
use anyhow::Result;
use mpl_token_metadata::accounts::Metadata;
use solana_sdk::{signature::Signature, signer::Signer};
use std::rc::Rc;

pub mod pda;
pub mod util;

/// Main client for interacting with the Zosh program
pub struct ZoshClient {
    /// Anchor client program instance
    program: Program<Rc<Keypair>>,

    /// Keypair of the payer
    keypair: Keypair,
}

impl ZoshClient {
    /// Create a new ZoshClient
    pub fn new(cluster_url: String, ws_url: String, payer: Keypair) -> Result<Self> {
        let secret = *payer.secret_bytes();
        let client = Client::new_with_options(
            Cluster::Custom(cluster_url, ws_url),
            Rc::new(payer),
            CommitmentConfig::confirmed(),
        );

        let program = client.program(crate::ID)?;
        Ok(Self {
            program,
            keypair: Keypair::new_from_array(secret),
        })
    }

    /// Get the payer's public key
    pub fn payer(&self) -> Pubkey {
        self.program.payer()
    }

    /// Get the program client
    pub fn program(&self) -> &Program<Rc<Keypair>> {
        &self.program
    }

    /// Sign a message with the payer's keypair
    pub fn sign_message(&self, message: &[u8]) -> Result<Signature> {
        let signature = self.keypair.sign_message(message);
        Ok(signature)
    }

    /// Read the current bridge state
    pub async fn bridge_state(&self) -> Result<crate::state::BridgeState> {
        let bridge_state = self.program.account(pda::bridge_state()).await?;
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

    /// Read the current zec balance for a recipient
    pub async fn zec_balance(&self, recipient: Pubkey) -> Result<TokenAccount> {
        let token_account = spl_associated_token_account::get_associated_token_address(
            &recipient,
            &pda::zec_mint(),
        );
        let account_data = self
            .program()
            .rpc()
            .get_account_data(&token_account)
            .await?;
        TokenAccount::try_deserialize(&mut &account_data[..]).map_err(Into::into)
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
        let mut builder = self.program.request();
        let state = self.bridge_state().await?;

        // create token accounts if not exists
        for entry in &mint_entries {
            let token_account = spl_associated_token_account::get_associated_token_address(
                &entry.recipient,
                &pda::zec_mint(),
            );
            if self
                .program()
                .rpc()
                .get_account_data(&token_account)
                .await
                .is_err()
            {
                let create_ata_ix =
                    spl_associated_token_account::instruction::create_associated_token_account(
                        &self.program().payer(),
                        &entry.recipient,
                        &pda::zec_mint(),
                        &pda::TOKEN_PROGRAM,
                    );
                builder = builder.instruction(create_ata_ix);
            }
        }

        // create the ed25519 verify instructions
        let message = util::create_mint_message(state.nonce, &mint_entries);
        for signature in &signatures {
            let ed25519_ix = solana_ed25519_program::new_ed25519_instruction_with_signature(
                &message,
                signature,
                &self.keypair.pubkey().to_bytes(),
            );
            builder = builder.instruction(ed25519_ix);
        }

        // create the mint instruction
        let bridge_state = pda::bridge_state();
        let zec_mint = pda::zec_mint();
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

        let _tx = builder
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
        validators: Vec<Pubkey>,
        threshold: u8,
        signatures: Vec<[u8; 64]>,
    ) -> Result<()> {
        anyhow::ensure!(!validators.is_empty(), "No validators provided");
        anyhow::ensure!(
            threshold > 0 && threshold as usize <= validators.len(),
            "Invalid threshold"
        );

        // Construct the ed25519 verify instruction
        let state = self.bridge_state().await?;
        let message = util::create_validators_message(state.nonce, &validators, threshold);
        let mut builder = self.program.request();
        for signature in &signatures {
            let signer_pubkey = self.keypair.pubkey();
            let ed25519_ix = solana_ed25519_program::new_ed25519_instruction_with_signature(
                &message,
                signature,
                &signer_pubkey.to_bytes(),
            );
            builder = builder.instruction(ed25519_ix);
        }

        // create the update instruction
        let bridge_state = pda::bridge_state();
        builder
            .accounts(crate::accounts::Validators {
                payer: self.program.payer(),
                bridge_state,
                system_program: pda::SYSTEM_PROGRAM,
                instructions: pda::INSTRUCTIONS_SYSVAR,
            })
            .args(crate::instruction::Validators {
                validators,
                threshold,
                signatures,
            })
            .send()
            .await?;

        eprintln!("[DEBUG] update_validators transaction sent successfully");
        Ok(())
    }
}
