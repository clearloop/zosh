//! client library for the zosh program
#![cfg(not(target_os = "solana"))]

use anchor_client::{
    solana_sdk::{commitment_config::CommitmentConfig, pubkey::Pubkey},
    Client, Cluster, Program,
};
use anchor_lang::{AccountDeserialize, InstructionData};
pub use anchor_lang::{AnchorDeserialize, Discriminator};
use anchor_spl::token::TokenAccount;
use anyhow::Result;
use mpl_token_metadata::accounts::Metadata;
use solana_sdk::{
    hash::Hash,
    signature::{Keypair, Signature},
    transaction::Transaction,
};
use std::rc::Rc;

pub mod pda;

/// Main client for interacting with the Zosh program
pub struct ZoshClient {
    /// Anchor client program instance
    pub program: Program<Rc<Keypair>>,
}

impl ZoshClient {
    /// Create a new ZoshClient
    pub fn new(cluster_url: String, ws_url: String, authority: Keypair) -> Result<Self> {
        let client = Client::new_with_options(
            Cluster::Custom(cluster_url, ws_url),
            Rc::new(authority),
            CommitmentConfig::confirmed(),
        );

        let program = client.program(crate::ID)?;
        Ok(Self { program })
    }

    /// Get the latest blockhash
    pub async fn latest_blockhash(&self) -> Result<Hash> {
        let hash = self.program.rpc().get_latest_blockhash().await?;
        Ok(hash)
    }

    /// Send a transaction
    pub async fn send(&self, tx: Transaction) -> Result<Signature> {
        let signature = self.program.rpc().send_transaction(&tx).await?;
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
        let account_data = self.program.rpc().get_account_data(&token_account).await?;
        TokenAccount::try_deserialize(&mut &account_data[..]).map_err(Into::into)
    }

    /// Initialize the bridge with initial validator set
    pub async fn initialize(&self, mpc: Pubkey) -> Result<Signature> {
        let bridge_state = pda::bridge_state();
        let zec_mint = pda::zec_mint();
        self.program
            .request()
            .accounts(crate::accounts::Initialize {
                payer: self.program.payer(),
                bridge_state,
                zec_mint,
                system_program: pda::SYSTEM_PROGRAM,
                token_program: pda::TOKEN_PROGRAM,
                rent: pda::RENT,
            })
            .args(crate::instruction::Initialize { mpc })
            .send()
            .await
            .map_err(Into::into)
    }

    /// Update token metadata (authority only)
    pub async fn update_metadata(
        &self,
        name: String,
        symbol: String,
        uri: String,
    ) -> Result<Signature> {
        let bridge_state = pda::bridge_state();
        let zec_mint = pda::zec_mint();
        let metadata = pda::metadata();
        self.program
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
            .await
            .map_err(Into::into)
    }

    /// Burn sZEC to bridge back to Zcash (public action)
    pub async fn burn(&self, amount: u64, zec_recipient: String) -> Result<Signature> {
        anyhow::ensure!(amount > 0, "Amount must be greater than 0");
        let bridge_state = pda::bridge_state();
        let zec_mint = pda::zec_mint();
        let signer_token_account = spl_associated_token_account::get_associated_token_address(
            &self.program.payer(),
            &zec_mint,
        );

        self.program
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
            .await
            .map_err(Into::into)
    }

    /// Update the MPC
    pub async fn update_mpc(&self, mpc: Pubkey) -> Result<Transaction> {
        let bridge_state = pda::bridge_state();
        let bridge_state_data = self.bridge_state().await?;

        // Build instruction manually to control the fee payer
        let account_metas = vec![
            solana_sdk::instruction::AccountMeta::new(bridge_state_data.mpc, true), // payer (signer)
            solana_sdk::instruction::AccountMeta::new(bridge_state, false),
            solana_sdk::instruction::AccountMeta::new_readonly(pda::SYSTEM_PROGRAM, false),
        ];

        // Serialize instruction data
        let data = crate::instruction::UpdateMpc { mpc }.data();

        let instruction = solana_sdk::instruction::Instruction {
            program_id: crate::ID,
            accounts: account_metas,
            data,
        };

        // Build transaction with MPC as fee payer
        let recent_blockhash = self.latest_blockhash().await?;
        let message = solana_sdk::message::Message::new_with_blockhash(
            &[instruction],
            Some(&bridge_state_data.mpc),
            &recent_blockhash,
        );

        Ok(Transaction {
            signatures: vec![solana_sdk::signature::Signature::default()],
            message,
        })
    }

    /// Mint sZEC to recipients (threshold action)
    pub async fn mint(&self, mints: Vec<crate::types::MintEntry>) -> Result<Transaction> {
        anyhow::ensure!(!mints.is_empty(), "No mint entries provided");
        let bridge_state = pda::bridge_state();
        let bridge_state_data = self.bridge_state().await?;
        let zec_mint = pda::zec_mint();

        // Build instruction manually to control the fee payer
        let mut account_metas = vec![
            solana_sdk::instruction::AccountMeta::new(bridge_state_data.mpc, true), // payer (signer)
            solana_sdk::instruction::AccountMeta::new(bridge_state, false),
            solana_sdk::instruction::AccountMeta::new(zec_mint, false),
            solana_sdk::instruction::AccountMeta::new_readonly(pda::TOKEN_PROGRAM, false),
            solana_sdk::instruction::AccountMeta::new_readonly(
                pda::ASSOCIATED_TOKEN_PROGRAM,
                false,
            ),
            solana_sdk::instruction::AccountMeta::new_readonly(pda::SYSTEM_PROGRAM, false),
        ];

        // Add ATAs and recipients as remaining accounts (pattern: ata, recipient, ata, recipient, ...)
        for entry in &mints {
            let token_account = spl_associated_token_account::get_associated_token_address(
                &entry.recipient,
                &pda::zec_mint(),
            );
            account_metas.push(solana_sdk::instruction::AccountMeta::new(
                token_account,
                false,
            ));
            account_metas.push(solana_sdk::instruction::AccountMeta::new_readonly(
                entry.recipient,
                false,
            ));
        }

        // Serialize instruction data using InstructionData
        let data = crate::instruction::Mint { mints }.data();
        let instruction = solana_sdk::instruction::Instruction {
            program_id: crate::ID,
            accounts: account_metas,
            data,
        };

        // Build transaction with MPC as fee payer
        let recent_blockhash = self.latest_blockhash().await?;
        let message = solana_sdk::message::Message::new_with_blockhash(
            &[instruction],
            Some(&bridge_state_data.mpc),
            &recent_blockhash,
        );

        Ok(Transaction {
            signatures: vec![solana_sdk::signature::Signature::default()],
            message,
        })
    }
}
