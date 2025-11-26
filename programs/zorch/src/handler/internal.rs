//! for the internal handlers of the zorch program

use crate::errors::BridgeError;
use anchor_lang::prelude::*;
use mpl_token_metadata::{
    instructions::{CreateV1CpiBuilder, UpdateV1CpiBuilder},
    types::{PrintSupply, TokenStandard},
};

/// Initialize the bridge with initial validator set
///
/// TODO: ensure this instruction should only be called for once.
pub fn initialize(
    ctx: Context<crate::Initialize>,
    validators: Vec<Pubkey>,
    threshold: u8,
) -> Result<()> {
    let total_validators = validators.len() as u8;
    require!(total_validators > 0, BridgeError::InvalidThreshold);
    require!(
        threshold > 0 && threshold <= total_validators,
        BridgeError::InvalidThreshold
    );

    let bridge_state = &mut ctx.accounts.bridge_state;
    bridge_state.authority = ctx.accounts.payer.key();
    bridge_state.validators = validators;
    bridge_state.threshold = threshold;
    bridge_state.total_validators = total_validators;
    bridge_state.nonce = 0;
    bridge_state.zec_mint = ctx.accounts.zec_mint.key();
    bridge_state.bump = ctx.bumps.bridge_state;
    msg!(
        "Bridge initialized with {} validators and threshold {}",
        total_validators,
        threshold
    );

    Ok(())
}

/// Update or create token metadata for the sZEC mint
pub fn metadata(
    ctx: Context<crate::UpdateMetadata>,
    name: String,
    symbol: String,
    uri: String,
) -> Result<()> {
    msg!("Updating token metadata: {} ({})", name, symbol);

    // Derive the metadata PDA
    let zec_mint_key = ctx.accounts.zec_mint.key();
    let metadata_seeds = &[
        b"metadata",
        mpl_token_metadata::ID.as_ref(),
        zec_mint_key.as_ref(),
    ];
    let (metadata_pda, _bump) =
        Pubkey::find_program_address(metadata_seeds, &mpl_token_metadata::ID);

    // Verify the provided metadata account matches the derived PDA
    require!(
        ctx.accounts.metadata.key() == metadata_pda,
        BridgeError::InvalidRecipient
    );

    // Check if metadata account exists
    let metadata_account_exists = ctx.accounts.metadata.data_len() > 0;
    let token_metadata_program = &ctx.accounts.token_metadata_program;
    let metadata_account = &ctx.accounts.metadata;
    let bridge_state_account = ctx.accounts.bridge_state.to_account_info();
    let zec_mint_account = ctx.accounts.zec_mint.to_account_info();
    let authority_account = &ctx.accounts.authority;
    let system_program_account = &ctx.accounts.system_program;
    let sysvar_instructions_account = &ctx.accounts.sysvar_instructions;
    if metadata_account_exists {
        let mut builder = UpdateV1CpiBuilder::new(token_metadata_program);
        builder
            .metadata(metadata_account)
            .authority(&bridge_state_account)
            .payer(authority_account)
            .system_program(system_program_account);

        // Build and invoke with PDA signer
        builder.invoke_signed(&[&[b"bridge-state", &[ctx.accounts.bridge_state.bump]]])?;
    } else {
        let mut builder = CreateV1CpiBuilder::new(token_metadata_program);
        builder
            .metadata(metadata_account)
            .mint(&zec_mint_account, false)
            .authority(&bridge_state_account)
            .payer(authority_account)
            .update_authority(&bridge_state_account, true)
            .system_program(system_program_account)
            .sysvar_instructions(sysvar_instructions_account)
            .name(name)
            .symbol(symbol)
            .uri(uri)
            .seller_fee_basis_points(0)
            .token_standard(TokenStandard::Fungible)
            .print_supply(PrintSupply::Zero);

        builder.invoke_signed(&[&[b"bridge-state", &[ctx.accounts.bridge_state.bump]]])?;
    }

    msg!("Token metadata updated successfully");
    Ok(())
}
