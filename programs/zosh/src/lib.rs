//! zosh consensus program

use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};
pub use errors::BridgeError;
pub use events::{BurnEvent, MintEvent};
use handler::{external, internal, threshold};
pub use state::BridgeState;

declare_id!("5QXepWTdHmsQkroWnitvs55jR6TxWbE8DCf54fQaYcH1");

pub mod client;
pub mod errors;
pub mod events;
mod handler;
pub mod state;
pub mod types;

#[program]
pub mod zosh {
    use super::*;

    /// Initialize the bridge with initial validator set and create sZEC mint
    pub fn initialize(ctx: Context<Initialize>, mpc: Pubkey) -> Result<()> {
        internal::initialize(ctx, mpc)
    }

    /// Update token metadata (internal action, authority only)
    pub fn metadata(
        ctx: Context<UpdateMetadata>,
        name: String,
        symbol: String,
        uri: String,
    ) -> Result<()> {
        internal::metadata(ctx, name, symbol, uri)
    }

    /// Mint sZEC to recipients (threshold action, supports batch)
    pub fn mint<'info>(
        ctx: Context<'_, '_, '_, 'info, MintZec<'info>>,
        mints: Vec<types::MintEntry>,
    ) -> Result<()> {
        threshold::mint(ctx, mints)
    }

    /// Burn sZEC to bridge back to Zcash (public action)
    pub fn burn(ctx: Context<BurnZec>, amount: u64, zec_recipient: String) -> Result<()> {
        external::burn(ctx, amount, zec_recipient)
    }

    /// Update the MPC pubkey (threshold action)
    pub fn update_mpc<'info>(
        ctx: Context<'_, '_, '_, 'info, UpdateMpc<'info>>,
        mpc: Pubkey,
    ) -> Result<()> {
        threshold::update_mpc(ctx, mpc)
    }
}

// ============================================================================
// Account Structs
// ============================================================================

/// Accounts for initializing the bridge.
#[derive(Accounts)]
#[instruction(mpc: Pubkey)]
pub struct Initialize<'info> {
    /// Transaction fee payer and rent payer for new accounts.
    #[account(mut)]
    pub payer: Signer<'info>,

    /// The main bridge state account storing validators and configuration.
    #[account(
        init,
        payer = payer,
        space = 8 + 32 + 32 + 32 + 1,
        seeds = [b"bridge-state"],
        bump
    )]
    pub bridge_state: Account<'info, BridgeState>,

    /// The sZEC SPL token mint.
    #[account(
        init,
        payer = payer,
        mint::decimals = 8,
        mint::authority = bridge_state,
        seeds = [b"zec-mint"],
        bump
    )]
    pub zec_mint: Account<'info, Mint>,

    /// System program for account creation.
    pub system_program: Program<'info, System>,

    /// Token program for mint creation.
    pub token_program: Program<'info, Token>,

    /// Rent sysvar for rent calculations.
    pub rent: Sysvar<'info, Rent>,
}

/// Accounts for updating token metadata.
#[derive(Accounts)]
pub struct UpdateMetadata<'info> {
    /// Bridge authority that can update metadata.
    ///
    /// Must match the authority stored in bridge_state.
    #[account(
        mut,
        constraint = authority.key() == bridge_state.authority @ BridgeError::InvalidRecipient
    )]
    pub authority: Signer<'info>,

    /// Bridge state for authority validation.
    #[account(
        seeds = [b"bridge-state"],
        bump = bridge_state.bump
    )]
    pub bridge_state: Account<'info, BridgeState>,

    /// The sZEC token mint.
    #[account(
        mut,
        seeds = [b"zec-mint"],
        bump,
        constraint = zec_mint.key() == bridge_state.zec_mint @ BridgeError::InvalidMint
    )]
    pub zec_mint: Account<'info, Mint>,

    /// Metaplex metadata account for the mint.
    ///
    /// CHECK: This is the metadata account for the mint.
    #[account(mut)]
    pub metadata: UncheckedAccount<'info>,

    /// Metaplex Token Metadata program.
    ///
    /// CHECK: This is the Metaplex Token Metadata program.
    pub token_metadata_program: UncheckedAccount<'info>,

    /// System program for account creation.
    pub system_program: Program<'info, System>,

    /// Sysvar instructions account required by mpl-token-metadata.
    ///
    /// CHECK: This is the sysvar instructions account required by mpl-token-metadata.
    pub sysvar_instructions: UncheckedAccount<'info>,
}

/// Accounts for burning sZEC tokens.
#[derive(Accounts)]
pub struct BurnZec<'info> {
    /// User burning their sZEC tokens.
    #[account(mut)]
    pub signer: Signer<'info>,

    /// User's token account holding sZEC to be burned.
    #[account(
        mut,
        constraint = signer_token_account.owner == signer.key() @ BridgeError::InvalidAmount,
        constraint = signer_token_account.mint == bridge_state.zec_mint @ BridgeError::InvalidAmount
    )]
    pub signer_token_account: Account<'info, TokenAccount>,

    /// The sZEC token mint.
    #[account(
        mut,
        constraint = zec_mint.key() == bridge_state.zec_mint @ BridgeError::InvalidAmount
    )]
    pub zec_mint: Account<'info, Mint>,

    /// Bridge state for mint validation.
    #[account(
        seeds = [b"bridge-state"],
        bump = bridge_state.bump
    )]
    pub bridge_state: Account<'info, BridgeState>,

    /// Token program for burn operation.
    pub token_program: Program<'info, Token>,
}

/// Accounts for minting sZEC tokens.
#[derive(Accounts)]
#[instruction(mints: Vec<types::MintEntry>)]
pub struct MintZec<'info> {
    /// Transaction fee payer.
    #[account(mut, constraint = payer.key() == bridge_state.mpc @ BridgeError::InvalidMpcSigner)]
    pub payer: Signer<'info>,

    /// Bridge state PDA storing validator set and configuration.
    #[account(
        mut,
        seeds = [b"bridge-state"],
        bump = bridge_state.bump
    )]
    pub bridge_state: Account<'info, BridgeState>,

    /// The sZEC token mint.
    #[account(
        mut,
        seeds = [b"zec-mint"],
        bump,
        constraint = zec_mint.key() == bridge_state.zec_mint @ BridgeError::InvalidMint
    )]
    pub zec_mint: Account<'info, Mint>,

    /// Token program for mint operations.
    pub token_program: Program<'info, Token>,

    /// Associated Token Account program.
    /// CHECK: This is the Associated Token Account program ID
    #[account(constraint = associated_token_program.key() == anchor_spl::associated_token::ID)]
    pub associated_token_program: AccountInfo<'info>,

    /// System program.
    pub system_program: Program<'info, System>,
}

/// Accounts for updating the MPC pubkey.
#[derive(Accounts)]
#[instruction(mpc: Pubkey)]
pub struct UpdateMpc<'info> {
    /// Transaction fee payer.
    #[account(mut)]
    pub payer: Signer<'info>,

    /// Bridge state PDA storing validator set and configuration.
    #[account(
        mut,
        seeds = [b"bridge-state"],
        bump = bridge_state.bump
    )]
    pub bridge_state: Account<'info, BridgeState>,

    /// System program.
    pub system_program: Program<'info, System>,
}
