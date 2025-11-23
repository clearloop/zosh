//! Threshold action handlers - operations that require validator signatures

use crate::{
    errors::BridgeError,
    events::{MintEvent, ValidatorSetUpdated},
    utils::{verify_threshold_signatures, ActionType},
};
use anchor_lang::prelude::*;
use anchor_spl::token::{self, MintTo};

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

/// Update the entire validator set (requires threshold signatures)
pub fn update_validators_full(
    ctx: Context<crate::UpdateValidatorsFull>,
    new_validators: Vec<Pubkey>,
    new_threshold: u16,
    signatures: Vec<[u8; 64]>,
) -> Result<()> {
    let bridge_state = &mut ctx.accounts.bridge_state;
    let new_total = new_validators.len() as u16;

    // Validate new threshold
    require!(
        new_threshold > 0 && new_threshold <= new_total,
        BridgeError::InvalidThreshold
    );
    require!(new_total > 0, BridgeError::InvalidThreshold);

    // Serialize action data for signature verification
    let mut action_data = Vec::new();
    action_data.extend_from_slice(&new_threshold.to_le_bytes());
    for validator in &new_validators {
        action_data.extend_from_slice(validator.as_ref());
    }

    // Verify threshold signatures from current validator set
    let _signers = verify_threshold_signatures(
        ActionType::UpdateValidatorsFull,
        bridge_state.nonce,
        &action_data,
        &signatures,
        &bridge_state.validators,
        bridge_state.threshold,
    )?;

    // Emit event
    emit!(ValidatorSetUpdated {
        old_validators: bridge_state.validators.clone(),
        new_validators: new_validators.clone(),
        threshold: new_threshold,
        nonce: bridge_state.nonce,
    });

    // Update the validator set
    bridge_state.validators = new_validators;
    bridge_state.threshold = new_threshold;
    bridge_state.total_validators = new_total;
    bridge_state.nonce += 1;

    msg!(
        "Validator set updated to {} validators with threshold {}",
        new_total,
        new_threshold
    );

    Ok(())
}

/// Add a single validator to the set (requires threshold signatures)
pub fn add_validator(
    ctx: Context<crate::AddValidator>,
    validator: Pubkey,
    signatures: Vec<[u8; 64]>,
) -> Result<()> {
    let bridge_state = &mut ctx.accounts.bridge_state;

    // Check if validator already exists
    require!(
        !bridge_state.validators.contains(&validator),
        BridgeError::ValidatorAlreadyExists
    );

    // Verify threshold signatures from current validator set
    let _signers = verify_threshold_signatures(
        ActionType::AddValidator,
        bridge_state.nonce,
        validator.as_ref(),
        &signatures,
        &bridge_state.validators,
        bridge_state.threshold,
    )?;

    // Add the validator
    let old_validators = bridge_state.validators.clone();
    bridge_state.validators.push(validator);
    bridge_state.total_validators += 1;
    bridge_state.nonce += 1;

    // Emit event
    emit!(ValidatorSetUpdated {
        old_validators,
        new_validators: bridge_state.validators.clone(),
        threshold: bridge_state.threshold,
        nonce: bridge_state.nonce - 1,
    });

    msg!("Validator added: {}", validator);

    Ok(())
}

/// Remove a single validator from the set (requires threshold signatures)
pub fn remove_validator(
    ctx: Context<crate::RemoveValidator>,
    validator: Pubkey,
    signatures: Vec<[u8; 64]>,
) -> Result<()> {
    let bridge_state = &mut ctx.accounts.bridge_state;

    // Check if validator exists
    require!(
        bridge_state.validators.contains(&validator),
        BridgeError::ValidatorNotFound
    );

    // Check that removing this validator won't violate threshold
    let new_total = bridge_state.total_validators - 1;
    require!(
        bridge_state.threshold <= new_total,
        BridgeError::CannotRemoveValidator
    );

    // Verify threshold signatures from current validator set
    let _signers = verify_threshold_signatures(
        ActionType::RemoveValidator,
        bridge_state.nonce,
        validator.as_ref(),
        &signatures,
        &bridge_state.validators,
        bridge_state.threshold,
    )?;

    // Remove the validator
    let old_validators = bridge_state.validators.clone();
    bridge_state.validators.retain(|v| v != &validator);
    bridge_state.total_validators -= 1;
    bridge_state.nonce += 1;

    // Emit event
    emit!(ValidatorSetUpdated {
        old_validators,
        new_validators: bridge_state.validators.clone(),
        threshold: bridge_state.threshold,
        nonce: bridge_state.nonce - 1,
    });

    msg!("Validator removed: {}", validator);

    Ok(())
}
