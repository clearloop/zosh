//! Threshold action handlers - operations that require validator signatures

use crate::{errors::BridgeError, events::MintEvent};
use anchor_lang::prelude::*;
use anchor_spl::token::{self, MintTo, TokenAccount};

/// Maximum number of mints allowed in a single batch
pub const MAX_BATCH_SIZE: usize = 10;

/// Mints sZEC tokens to recipients (supports batch).
pub fn mint<'info>(
    ctx: Context<'_, '_, '_, 'info, crate::MintZec<'info>>,
    mints: Vec<crate::types::MintEntry>,
) -> Result<()> {
    require!(
        !mints.is_empty() && mints.len() <= MAX_BATCH_SIZE,
        BridgeError::InvalidBatchSize
    );

    // Process each mint in the batch
    let bridge_state_bump = ctx.accounts.bridge_state.bump;
    let mut mint_tuples = Vec::with_capacity(mints.len());
    let zec_mint_key = ctx.accounts.zec_mint.key();
    let seeds = &[b"bridge-state".as_ref(), &[bridge_state_bump]];
    let signer_seeds = &[&seeds[..]];

    // We need recipient accounts in remaining_accounts after the ATAs
    // Remaining accounts layout: [ata0, recipient0, ata1, recipient1, ...]
    require!(
        ctx.remaining_accounts.len() == mints.len() * 2,
        BridgeError::InvalidRecipient
    );

    for (i, mint_entry) in mints.iter().enumerate() {
        let recipient_token_account_info = &ctx.remaining_accounts[i * 2];
        let recipient_account_info = &ctx.remaining_accounts[i * 2 + 1];
        require!(
            recipient_account_info.key() == mint_entry.recipient,
            BridgeError::InvalidRecipient
        );

        // Check if the ATA exists, create it if not
        if recipient_token_account_info.data_is_empty() {
            let cpi_accounts = anchor_spl::associated_token::Create {
                payer: ctx.accounts.payer.to_account_info(),
                associated_token: recipient_token_account_info.to_account_info(),
                authority: recipient_account_info.to_account_info(),
                mint: ctx.accounts.zec_mint.to_account_info(),
                system_program: ctx.accounts.system_program.to_account_info(),
                token_program: ctx.accounts.token_program.to_account_info(),
            };
            let cpi_program = ctx.accounts.associated_token_program.to_account_info();
            let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
            anchor_spl::associated_token::create(cpi_ctx)?;
        } else {
            let token_account_data = recipient_token_account_info.try_borrow_data()?;
            let token_account = TokenAccount::try_deserialize(&mut &token_account_data[..])?;
            require!(token_account.mint == zec_mint_key, BridgeError::InvalidMint);
            require!(
                token_account.owner == mint_entry.recipient,
                BridgeError::InvalidRecipient
            );
            drop(token_account_data);
        }

        // Mint tokens to this recipient
        let cpi_accounts = MintTo {
            mint: ctx.accounts.zec_mint.to_account_info(),
            to: recipient_token_account_info.to_account_info(),
            authority: ctx.accounts.bridge_state.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer_seeds);
        token::mint_to(cpi_ctx, mint_entry.amount)?;
        mint_tuples.push((mint_entry.recipient, mint_entry.amount));
    }

    // Emit batch event
    emit!(MintEvent {
        mints: mint_tuples,
        timestamp: Clock::get()?.unix_timestamp,
    });

    Ok(())
}

/// Updates the MPC pubkey in the bridge state.
pub fn update_mpc<'info>(
    ctx: Context<'_, '_, '_, 'info, crate::UpdateMpc<'info>>,
    new_mpc: Pubkey,
) -> Result<()> {
    // Verify that the signer is the MPC
    require!(
        ctx.accounts.payer.key() == ctx.accounts.bridge_state.mpc,
        BridgeError::InvalidMpcSigner
    );

    let bridge_state = &mut ctx.accounts.bridge_state;
    bridge_state.mpc = new_mpc;
    Ok(())
}
