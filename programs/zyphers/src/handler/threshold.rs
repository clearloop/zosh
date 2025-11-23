//! Threshold action handlers - operations that require validator signatures

use anchor_lang::prelude::*;
use anchor_spl::token::{self, MintTo};

use crate::errors::BridgeError;
use crate::events::MintEvent;
use crate::utils::{verify_threshold_signatures, ActionType};


/// Mint sZEC to a recipient (requires threshold signatures)
pub fn mint(
    ctx: Context<crate::MintSzec>,
    recipient: Pubkey,
    amount: u64,
    signatures: Vec<[u8; 64]>,
) -> Result<()> {
    require!(amount > 0, BridgeError::InvalidAmount);

    // Serialize action data for signature verification
    let mut action_data = Vec::new();
    action_data.extend_from_slice(recipient.as_ref());
    action_data.extend_from_slice(&amount.to_le_bytes());

    // Get references for verification
    let nonce = ctx.accounts.bridge_state.nonce;
    let validators = ctx.accounts.bridge_state.validators.clone();
    let threshold = ctx.accounts.bridge_state.threshold;
    let bridge_state_bump = ctx.accounts.bridge_state.bump;

    // Verify threshold signatures from current validator set
    let _signers = verify_threshold_signatures(
        ActionType::Mint,
        nonce,
        &action_data,
        &signatures,
        &validators,
        threshold,
    )?;

    // Mint sZEC tokens
    let seeds = &[b"bridge-state".as_ref(), &[bridge_state_bump]];
    let signer_seeds = &[&seeds[..]];

    let cpi_accounts = MintTo {
        mint: ctx.accounts.szec_mint.to_account_info(),
        to: ctx.accounts.recipient_token_account.to_account_info(),
        authority: ctx.accounts.bridge_state.to_account_info(),
    };
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);

    token::mint_to(cpi_ctx, amount)?;

    // Emit event
    emit!(MintEvent {
        recipient,
        amount,
        nonce,
        timestamp: Clock::get()?.unix_timestamp,
    });

    // Increment nonce
    ctx.accounts.bridge_state.nonce += 1;

    msg!("Minted {} sZEC to {}", amount, recipient);

    Ok(())
}
