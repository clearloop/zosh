//! for the internal handlers of the zyphers program

use crate::errors::BridgeError;
use anchor_lang::prelude::*;

/// Initialize the bridge with initial validator set
///
/// TODO: ensure this instruction should only be called for once.
pub fn initialize(
    ctx: Context<crate::Initialize>,
    initial_validators: Vec<Pubkey>,
    threshold: u8,
) -> Result<()> {
    let total_validators = initial_validators.len() as u8;
    require!(total_validators > 0, BridgeError::InvalidThreshold);
    require!(
        threshold > 0 && threshold <= total_validators,
        BridgeError::InvalidThreshold
    );

    let bridge_state = &mut ctx.accounts.bridge_state;
    bridge_state.authority = ctx.accounts.payer.key();
    bridge_state.validators = initial_validators;
    bridge_state.threshold = threshold;
    bridge_state.total_validators = total_validators;
    bridge_state.nonce = 0;
    bridge_state.szec_mint = ctx.accounts.szec_mint.key();
    bridge_state.bump = ctx.bumps.bridge_state;
    msg!(
        "Bridge initialized with {} validators and threshold {}",
        total_validators,
        threshold
    );

    Ok(())
}
