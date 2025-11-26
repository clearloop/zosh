//! Threshold action handlers - operations that require validator signatures

use crate::{
    errors::BridgeError,
    events::{MintEvent, ValidatorSetUpdated},
    utils::{verify_threshold_signatures, ActionType},
};
use anchor_lang::prelude::*;
use anchor_spl::token::{self, MintTo, TokenAccount};

/// Maximum number of mints allowed in a single batch
pub const MAX_BATCH_SIZE: usize = 10;

/// Mints sZEC tokens to recipients (supports batch).
pub fn mint<'info>(
    ctx: Context<'_, '_, '_, 'info, crate::MintSzec<'info>>,
    mints: Vec<(Pubkey, u64)>,
    signatures: Vec<[u8; 64]>,
) -> Result<()> {
    require!(
        !mints.is_empty() && mints.len() <= MAX_BATCH_SIZE,
        BridgeError::InvalidBatchSize
    );

    // Validate all amounts and compute total
    let mut action_data = Vec::new();
    for (recipient, amount) in &mints {
        require!(*amount > 0, BridgeError::InvalidAmount);
        action_data.extend_from_slice(recipient.as_ref());
        action_data.extend_from_slice(&amount.to_le_bytes());
    }

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
        &ctx.accounts.instructions,
    )?;

    // Verify we have the correct number of remaining accounts
    require!(
        ctx.remaining_accounts.len() == mints.len(),
        BridgeError::InvalidAccountCount
    );

    // Process each mint in the batch
    let mut recipients = Vec::with_capacity(mints.len());
    let mut amounts = Vec::with_capacity(mints.len());
    let zec_mint_key = ctx.accounts.zec_mint.key();
    let seeds = &[b"bridge-state".as_ref(), &[bridge_state_bump]];
    let signer_seeds = &[&seeds[..]];
    for (i, (recipient, amount)) in mints.iter().enumerate() {
        let recipient_token_account_info = &ctx.remaining_accounts[i];
        let token_account_data = recipient_token_account_info.try_borrow_data()?;
        let token_account = TokenAccount::try_deserialize(&mut &token_account_data[..])?;
        require!(token_account.mint == zec_mint_key, BridgeError::InvalidMint);
        require!(
            token_account.owner == *recipient,
            BridgeError::InvalidRecipient
        );

        // Mint tokens to this recipient
        drop(token_account_data);
        let cpi_accounts = MintTo {
            mint: ctx.accounts.zec_mint.to_account_info(),
            to: recipient_token_account_info.to_account_info(),
            authority: ctx.accounts.bridge_state.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);
        token::mint_to(cpi_ctx, *amount)?;
        recipients.push(*recipient);
        amounts.push(*amount);
    }

    // Emit batch event
    emit!(MintEvent {
        mints,
        nonce,
        timestamp: Clock::get()?.unix_timestamp,
    });

    // Increment nonce once for entire batch
    ctx.accounts.bridge_state.nonce += 1;
    Ok(())
}

/// Updates the entire validator set.
pub fn validators(
    ctx: Context<crate::Validators>,
    new_validators: Vec<Pubkey>,
    new_threshold: u8,
    signatures: Vec<[u8; 64]>,
) -> Result<()> {
    let bridge_state = &mut ctx.accounts.bridge_state;
    let new_total = new_validators.len() as u8;

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
        ActionType::Validators,
        bridge_state.nonce,
        &action_data,
        &signatures,
        &bridge_state.validators,
        bridge_state.threshold,
        &ctx.accounts.instructions,
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
