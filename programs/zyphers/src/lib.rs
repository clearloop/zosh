use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};

declare_id!("2KwobV7wjmUzGRQfpd3G5HVRfCRUXfry9MoM3Hbks9dz");

// Public modules
pub mod errors;
pub mod events;
pub mod state;

// Internal modules
mod external;
mod threshold;
mod utils;

// Re-export for external use
pub use errors::BridgeError;
pub use events::{BurnEvent, MintEvent, ValidatorSetUpdated};
pub use state::{ActionRecord, BridgeState};

#[program]
pub mod zyphers {
    use super::*;

    /// Initialize the bridge with initial validator set and create sZEC mint
    pub fn initialize(
        ctx: Context<Initialize>,
        initial_validators: Vec<Pubkey>,
        threshold: u16,
    ) -> Result<()> {
        threshold::initialize(ctx, initial_validators, threshold)
    }

    /// Mint sZEC to a recipient (threshold action)
    pub fn mint(
        ctx: Context<MintSzec>,
        recipient: Pubkey,
        amount: u64,
        signatures: Vec<[u8; 64]>,
    ) -> Result<()> {
        threshold::mint(ctx, recipient, amount, signatures)
    }

    /// Burn sZEC to bridge back to Zcash (public action)
    pub fn burn(ctx: Context<BurnSzec>, amount: u64, zec_recipient: String) -> Result<()> {
        external::burn(ctx, amount, zec_recipient)
    }
}

// ============================================================================
// Account Structs
// ============================================================================

#[derive(Accounts)]
#[instruction(initial_validators: Vec<Pubkey>, threshold: u16)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        init,
        payer = payer,
        space = BridgeState::space(initial_validators.len()),
        seeds = [b"bridge-state"],
        bump
    )]
    pub bridge_state: Account<'info, BridgeState>,

    #[account(
        init,
        payer = payer,
        mint::decimals = 8,
        mint::authority = bridge_state,
        seeds = [b"szec-mint"],
        bump
    )]
    pub szec_mint: Account<'info, Mint>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
#[instruction(recipient: Pubkey, amount: u64, signatures: Vec<[u8; 64]>)]
pub struct MintSzec<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        mut,
        seeds = [b"bridge-state"],
        bump = bridge_state.bump
    )]
    pub bridge_state: Account<'info, BridgeState>,

    #[account(
        mut,
        seeds = [b"szec-mint"],
        bump,
        constraint = szec_mint.key() == bridge_state.szec_mint @ BridgeError::InvalidAmount
    )]
    pub szec_mint: Account<'info, Mint>,

    #[account(
        mut,
        constraint = recipient_token_account.mint == szec_mint.key() @ BridgeError::InvalidAmount,
        constraint = recipient_token_account.owner == recipient @ BridgeError::InvalidAmount
    )]
    pub recipient_token_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct BurnSzec<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        mut,
        constraint = signer_token_account.owner == signer.key() @ BridgeError::InvalidAmount,
        constraint = signer_token_account.mint == bridge_state.szec_mint @ BridgeError::InvalidAmount
    )]
    pub signer_token_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = szec_mint.key() == bridge_state.szec_mint @ BridgeError::InvalidAmount
    )]
    pub szec_mint: Account<'info, Mint>,

    #[account(
        seeds = [b"bridge-state"],
        bump = bridge_state.bump
    )]
    pub bridge_state: Account<'info, BridgeState>,

    pub token_program: Program<'info, Token>,
}
