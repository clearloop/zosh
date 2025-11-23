//! zyphers consensus program

use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};
pub use errors::BridgeError;
pub use events::{BurnEvent, MintEvent, ValidatorSetUpdated};
use handler::{external, internal, threshold};
pub use state::{ActionRecord, BridgeState};

declare_id!("2KwobV7wjmUzGRQfpd3G5HVRfCRUXfry9MoM3Hbks9dz");

pub mod errors;
pub mod events;
mod handler;
pub mod state;
mod utils;

#[program]
pub mod zyphers {
    use super::*;

    /// Initialize the bridge with initial validator set and create sZEC mint
    pub fn initialize(
        ctx: Context<Initialize>,
        initial_validators: Vec<Pubkey>,
        threshold: u16,
    ) -> Result<()> {
        internal::initialize(ctx, initial_validators, threshold)
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

    /// Update the entire validator set (threshold action)
    pub fn update_validators_full(
        ctx: Context<UpdateValidatorsFull>,
        new_validators: Vec<Pubkey>,
        new_threshold: u16,
        signatures: Vec<[u8; 64]>,
    ) -> Result<()> {
        threshold::update_validators_full(ctx, new_validators, new_threshold, signatures)
    }

    /// Add a single validator to the set (threshold action)
    pub fn add_validator(
        ctx: Context<AddValidator>,
        validator: Pubkey,
        signatures: Vec<[u8; 64]>,
    ) -> Result<()> {
        threshold::add_validator(ctx, validator, signatures)
    }

    /// Remove a single validator from the set (threshold action)
    pub fn remove_validator(
        ctx: Context<RemoveValidator>,
        validator: Pubkey,
        signatures: Vec<[u8; 64]>,
    ) -> Result<()> {
        threshold::remove_validator(ctx, validator, signatures)
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

#[derive(Accounts)]
#[instruction(new_validators: Vec<Pubkey>, new_threshold: u16, signatures: Vec<[u8; 64]>)]
pub struct UpdateValidatorsFull<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        mut,
        seeds = [b"bridge-state"],
        bump = bridge_state.bump,
        realloc = BridgeState::space(new_validators.len()),
        realloc::payer = payer,
        realloc::zero = false
    )]
    pub bridge_state: Account<'info, BridgeState>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(validator: Pubkey, signatures: Vec<[u8; 64]>)]
pub struct AddValidator<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        mut,
        seeds = [b"bridge-state"],
        bump = bridge_state.bump,
        realloc = BridgeState::space(bridge_state.validators.len() + 1),
        realloc::payer = payer,
        realloc::zero = false
    )]
    pub bridge_state: Account<'info, BridgeState>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(validator: Pubkey, signatures: Vec<[u8; 64]>)]
pub struct RemoveValidator<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        mut,
        seeds = [b"bridge-state"],
        bump = bridge_state.bump,
        realloc = BridgeState::space(bridge_state.validators.len().saturating_sub(1)),
        realloc::payer = payer,
        realloc::zero = false
    )]
    pub bridge_state: Account<'info, BridgeState>,

    pub system_program: Program<'info, System>,
}
