//! Solana client for the Zorch bridge program built on anchor-client

pub mod signatures;

use anchor_client::{
    solana_sdk::{commitment_config::CommitmentConfig, pubkey::Pubkey, signature::Keypair},
    Client, Cluster, Program,
};
use anyhow::Result;
use std::rc::Rc;

/// Main client for interacting with the Zorch program
pub struct ZorchClient {
    /// Anchor client program instance
    program: Program<Rc<Keypair>>,
    /// The Zorch program ID
    program_id: Pubkey,
}

impl ZorchClient {
    /// Create a new ZorchClient
    ///
    /// # Arguments
    /// * `cluster_url` - The Solana RPC endpoint URL
    /// * `ws_url` - The Solana WebSocket endpoint URL
    /// * `payer` - The default payer keypair
    /// * `program_id` - The Zorch program ID
    pub fn new(
        cluster_url: String,
        ws_url: String,
        payer: Keypair,
        program_id: Pubkey,
    ) -> Result<Self> {
        let client = Client::new_with_options(
            Cluster::Custom(cluster_url, ws_url),
            Rc::new(payer),
            CommitmentConfig::confirmed(),
        );

        let program = client.program(program_id)?;

        Ok(Self {
            program,
            program_id,
        })
    }

    /// Create a new ZorchClient with custom commitment level
    pub fn new_with_commitment(
        cluster_url: String,
        ws_url: String,
        payer: Keypair,
        program_id: Pubkey,
        commitment: CommitmentConfig,
    ) -> Result<Self> {
        let client = Client::new_with_options(
            Cluster::Custom(cluster_url, ws_url),
            Rc::new(payer),
            commitment,
        );

        let program = client.program(program_id)?;

        Ok(Self {
            program,
            program_id,
        })
    }

    /// Get the program ID
    pub fn program_id(&self) -> &Pubkey {
        &self.program_id
    }

    /// Get a reference to the anchor-client Program
    pub fn program(&self) -> &Program<Rc<Keypair>> {
        &self.program
    }

    /// Get the payer pubkey
    pub fn payer(&self) -> Pubkey {
        self.program.payer()
    }

    // ============================================================================
    // PDA Helper Methods
    // ============================================================================

    /// Get the bridge state PDA address
    pub fn get_bridge_state_address(&self) -> Pubkey {
        let (address, _) = Pubkey::find_program_address(&[b"bridge-state"], &self.program_id);
        address
    }

    /// Get the sZEC mint PDA address
    pub fn get_zec_mint_address(&self) -> Pubkey {
        let (address, _) = Pubkey::find_program_address(&[b"zec-mint"], &self.program_id);
        address
    }

    /// Get the metadata PDA address (Metaplex)
    pub fn get_metadata_address(&self) -> Pubkey {
        let zec_mint = self.get_zec_mint_address();
        let metadata_program = mpl_token_metadata::ID;
        let (address, _) = Pubkey::find_program_address(
            &[b"metadata", metadata_program.as_ref(), zec_mint.as_ref()],
            &metadata_program,
        );
        address
    }

    // ============================================================================
    // State Reading Methods
    // ============================================================================

    /// Fetch and deserialize the BridgeState account
    pub fn get_bridge_state(&self) -> Result<zorch::BridgeState> {
        let bridge_state_address = self.get_bridge_state_address();
        self.program
            .account(bridge_state_address)
            .map_err(Into::into)
    }

    /// Get the current nonce from the bridge state
    pub fn get_current_nonce(&self) -> Result<u64> {
        let bridge_state: zorch::BridgeState = self.get_bridge_state()?;
        Ok(bridge_state.nonce)
    }

    /// Get the current validator set from the bridge state
    pub fn get_validators(&self) -> Result<Vec<Pubkey>> {
        let bridge_state: zorch::BridgeState = self.get_bridge_state()?;
        Ok(bridge_state.validators)
    }

    /// Get the current threshold from the bridge state
    pub fn get_threshold(&self) -> Result<u8> {
        let bridge_state: zorch::BridgeState = self.get_bridge_state()?;
        Ok(bridge_state.threshold)
    }

    /// Check if an address is a validator in the current set
    pub fn is_validator(&self, address: &Pubkey) -> Result<bool> {
        let validators = self.get_validators()?;
        Ok(validators.contains(address))
    }

    // ============================================================================
    // Transaction Methods
    // ============================================================================

    /// Initialize the bridge with initial validator set
    ///
    /// # Arguments
    /// * `validators` - Initial validator set
    /// * `threshold` - Number of validators required for consensus
    pub fn initialize(&self, validators: Vec<Pubkey>, threshold: u8) -> Result<Pubkey> {
        let bridge_state = self.get_bridge_state_address();
        let zec_mint = self.get_zec_mint_address();

        let tx = self
            .program
            .request()
            .accounts(zorch::accounts::Initialize {
                payer: self.payer(),
                bridge_state,
                zec_mint,
                system_program: anchor_client::solana_sdk::system_program::ID,
                token_program: spl_token::ID,
                rent: anchor_client::solana_sdk::sysvar::rent::ID,
            })
            .args(zorch::instruction::Initialize {
                validators,
                threshold,
            })
            .send()?;

        println!("Initialize transaction: {}", tx);
        Ok(bridge_state)
    }

    /// Update token metadata (authority only)
    ///
    /// # Arguments
    /// * `name` - Token name
    /// * `symbol` - Token symbol
    /// * `uri` - Metadata URI
    pub fn update_metadata(&self, name: String, symbol: String, uri: String) -> Result<()> {
        let bridge_state = self.get_bridge_state_address();
        let zec_mint = self.get_zec_mint_address();
        let metadata = self.get_metadata_address();

        let _tx = self
            .program
            .request()
            .accounts(zorch::accounts::UpdateMetadata {
                authority: self.payer(),
                bridge_state,
                zec_mint,
                metadata,
                token_metadata_program: mpl_token_metadata::ID,
                system_program: anchor_client::solana_sdk::system_program::ID,
                rent: anchor_client::solana_sdk::sysvar::rent::ID,
            })
            .args(zorch::instruction::Metadata { name, symbol, uri })
            .send()?;

        Ok(())
    }

    /// Mint sZEC to recipients (threshold action)
    ///
    /// # Arguments
    /// * `mint_entries` - Recipients and amounts to mint
    /// * `validator_keypairs` - Validator keypairs for signing (must meet threshold)
    ///
    /// Note: This requires building Ed25519 verification instructions manually.
    /// Use the signature helpers in this client to create the required signatures.
    pub fn send_mint(
        &self,
        mint_entries: Vec<zorch::types::MintEntry>,
        signatures: Vec<[u8; 64]>,
    ) -> Result<()> {
        anyhow::ensure!(!mint_entries.is_empty(), "No mint entries provided");

        let bridge_state = self.get_bridge_state_address();
        let zec_mint = self.get_zec_mint_address();

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
            .accounts(zorch::accounts::MintZec {
                payer: self.payer(),
                bridge_state,
                zec_mint,
                token_program: spl_token::ID,
                system_program: anchor_client::solana_sdk::system_program::ID,
                instructions: anchor_client::solana_sdk::sysvar::instructions::ID,
            })
            .args(zorch::instruction::Mint {
                mints: mint_entries,
                signatures,
            })
            .accounts(remaining_accounts)
            .send()?;

        Ok(())
    }

    /// Burn sZEC to bridge back to Zcash (public action)
    ///
    /// # Arguments
    /// * `amount` - Amount to burn
    /// * `zec_recipient` - Zcash address to receive ZEC
    pub fn send_burn(&self, amount: u64, zec_recipient: String) -> Result<()> {
        anyhow::ensure!(amount > 0, "Amount must be greater than 0");

        let bridge_state = self.get_bridge_state_address();
        let zec_mint = self.get_zec_mint_address();
        let signer_token_account =
            spl_associated_token_account::get_associated_token_address(&self.payer(), &zec_mint);

        let _tx = self
            .program
            .request()
            .accounts(zorch::accounts::BurnZec {
                signer: self.payer(),
                signer_token_account,
                zec_mint,
                bridge_state,
                token_program: spl_token::ID,
            })
            .args(zorch::instruction::Burn {
                amount,
                zec_recipient,
            })
            .send()?;

        Ok(())
    }

    /// Update the validator set (threshold action)
    ///
    /// # Arguments
    /// * `new_validators` - New validator set
    /// * `new_threshold` - New threshold requirement
    /// * `signatures` - Signatures from current validators (must meet current threshold)
    ///
    /// Note: This requires building Ed25519 verification instructions manually.
    /// Use the signature helpers in this client to create the required signatures.
    pub fn update_validators(
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

        let bridge_state = self.get_bridge_state_address();

        let _tx = self
            .program
            .request()
            .accounts(zorch::accounts::Validators {
                payer: self.payer(),
                bridge_state,
                system_program: anchor_client::solana_sdk::system_program::ID,
                instructions: anchor_client::solana_sdk::sysvar::instructions::ID,
            })
            .args(zorch::instruction::Validators {
                new_validators,
                new_threshold,
                signatures,
            })
            .send()?;

        Ok(())
    }
}
