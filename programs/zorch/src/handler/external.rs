//! External/public action handlers - operations anyone can submit

use crate::{errors::BridgeError, events::BurnEvent};
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Burn};

/// Burn sZEC to bridge back to Zcash (public action)
pub fn burn(ctx: Context<crate::BurnZec>, amount: u64, zec_recipient: String) -> Result<()> {
    require!(amount > 0, BridgeError::InvalidAmount);
    require!(
        !zec_recipient.is_empty() && zec_recipient.len() >= 26 && zec_recipient.len() <= 95,
        BridgeError::InvalidZcashAddress
    );

    // Burn sZEC tokens from signer's account
    let cpi_accounts = Burn {
        mint: ctx.accounts.zec_mint.to_account_info(),
        from: ctx.accounts.signer_token_account.to_account_info(),
        authority: ctx.accounts.signer.to_account_info(),
    };
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
    token::burn(cpi_ctx, amount)?;

    // Emit event for off-chain validators to monitor
    emit!(BurnEvent {
        sender: ctx.accounts.signer.key(),
        amount,
        zec_recipient: zec_recipient.clone(),
        timestamp: Clock::get()?.unix_timestamp,
    });

    msg!(
        "Burned {} sZEC from {} to ZEC address {}",
        amount,
        ctx.accounts.signer.key(),
        zec_recipient
    );

    Ok(())
}
