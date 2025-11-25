//! zorch consensus program

use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};
pub use errors::BridgeError;
pub use events::{BurnEvent, MintEvent, ValidatorSetUpdated};
use handler::{external, internal, threshold};
pub use state::{ActionRecord, BridgeState};

declare_id!("2KwobV7wjmUzGRQfpd3G5HVRfCRUXfry9MoM3Hbks9dz");

pub mod api;
pub mod errors;
pub mod events;
mod handler;
pub mod state;
mod utils;

#[program]
pub mod zorch {
    use super::*;

    /// Initialize the bridge with initial validator set and create sZEC mint
    pub fn initialize(
        ctx: Context<Initialize>,
        validators: Vec<Pubkey>,
        threshold: u8,
    ) -> Result<()> {
        internal::initialize(ctx, validators, threshold)
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
        new_threshold: u8,
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

/// Accounts for initializing the bridge.
///
/// This instruction creates the bridge state account and the sZEC SPL token mint.
/// It sets up the initial validator set and threshold for the consensus mechanism.
///
/// # Accounts
/// - `payer`: Transaction fee payer and rent payer for new accounts
/// - `bridge_state`: The main bridge state PDA that stores validator set and configuration
/// - `szec_mint`: The SPL token mint for sZEC with 8 decimals (matching ZEC)
/// - `system_program`: Required for account creation
/// - `token_program`: Required for mint creation
/// - `rent`: Rent sysvar for account rent calculations
#[derive(Accounts)]
#[instruction(validators: Vec<Pubkey>, threshold: u8)]
pub struct Initialize<'info> {
    /// Transaction fee payer and rent payer for new accounts.
    ///
    /// Must sign the transaction.
    #[account(mut)]
    pub payer: Signer<'info>,

    /// The main bridge state account storing validators and configuration.
    ///
    /// Initialized as a PDA with seeds `[b"bridge-state"]`.
    /// Space is calculated based on the number of initial validators.
    #[account(
        init,
        payer = payer,
        space = BridgeState::space(validators.len()),
        seeds = [b"bridge-state"],
        bump
    )]
    pub bridge_state: Account<'info, BridgeState>,

    /// The sZEC SPL token mint.
    ///
    /// Initialized with:
    /// - 8 decimals (matching ZEC)
    /// - Mint authority set to bridge_state PDA
    /// - Seeds `[b"szec-mint"]` for deterministic address
    #[account(
        init,
        payer = payer,
        mint::decimals = 8,
        mint::authority = bridge_state,
        seeds = [b"szec-mint"],
        bump
    )]
    pub szec_mint: Account<'info, Mint>,

    /// System program for account creation.
    pub system_program: Program<'info, System>,

    /// Token program for mint creation.
    pub token_program: Program<'info, Token>,

    /// Rent sysvar for rent calculations.
    pub rent: Sysvar<'info, Rent>,
}

/// Accounts for minting sZEC tokens.
///
/// This is a threshold action that requires signatures from validators meeting
/// the threshold requirement. Validators sign off-chain and provide signatures
/// to authorize the mint operation.
///
/// # Accounts
/// - `payer`: Transaction fee payer
/// - `bridge_state`: Stores validator set and is used as mint authority
/// - `szec_mint`: The sZEC token mint
/// - `recipient_token_account`: Recipient's token account to receive minted sZEC
/// - `token_program`: Required for minting
/// - `system_program`: Required for various operations
///
/// # Constraints
/// - Recipient token account must be for the sZEC mint
/// - Recipient token account must be owned by the specified recipient
#[derive(Accounts)]
#[instruction(recipient: Pubkey, amount: u64, signatures: Vec<[u8; 64]>)]
pub struct MintSzec<'info> {
    /// Transaction fee payer.
    ///
    /// Must sign the transaction.
    #[account(mut)]
    pub payer: Signer<'info>,

    /// Bridge state PDA storing validator set and configuration.
    ///
    /// Used as the mint authority for sZEC.
    /// Nonce is incremented after successful mint.
    #[account(
        mut,
        seeds = [b"bridge-state"],
        bump = bridge_state.bump
    )]
    pub bridge_state: Account<'info, BridgeState>,

    /// The sZEC token mint.
    ///
    /// Must match the mint stored in bridge_state.
    #[account(
        mut,
        seeds = [b"szec-mint"],
        bump,
        constraint = szec_mint.key() == bridge_state.szec_mint @ BridgeError::InvalidMint
    )]
    pub szec_mint: Account<'info, Mint>,

    /// Recipient's token account to receive minted sZEC.
    ///
    /// Must be:
    /// - For the sZEC mint
    /// - Owned by the specified recipient pubkey
    #[account(
        mut,
        constraint = recipient_token_account.mint == szec_mint.key() @ BridgeError::InvalidMint,
        constraint = recipient_token_account.owner == recipient @ BridgeError::InvalidRecipient
    )]
    pub recipient_token_account: Account<'info, TokenAccount>,

    /// Token program for mint operations.
    pub token_program: Program<'info, Token>,

    /// System program.
    pub system_program: Program<'info, System>,

    /// Instructions sysvar for Ed25519 signature verification.
    ///
    /// Used to read Ed25519 program verification results.
    /// Ed25519 instructions must be included before this instruction in the transaction.
    ///
    /// CHECK: Must be the Instructions sysvar account
    #[account(address = anchor_lang::solana_program::sysvar::instructions::ID)]
    pub instructions: UncheckedAccount<'info>,
}

/// Accounts for burning sZEC tokens.
///
/// This is a public action that anyone can perform to bridge their sZEC back
/// to ZEC on the Zcash network. The burn operation emits an event that off-chain
/// validators monitor to process the corresponding ZEC transfer.
///
/// # Accounts
/// - `signer`: User burning their sZEC tokens
/// - `signer_token_account`: User's token account holding sZEC
/// - `szec_mint`: The sZEC token mint (supply will decrease)
/// - `bridge_state`: Read-only reference for mint validation
/// - `token_program`: Required for burn operation
///
/// # Constraints
/// - Signer must own the token account
/// - Token account must hold sZEC tokens
/// - Mint must match bridge state's recorded mint
#[derive(Accounts)]
pub struct BurnSzec<'info> {
    /// User burning their sZEC tokens.
    ///
    /// Must sign the transaction and own the token account.
    #[account(mut)]
    pub signer: Signer<'info>,

    /// User's token account holding sZEC to be burned.
    ///
    /// Must be:
    /// - Owned by the signer
    /// - For the sZEC mint
    #[account(
        mut,
        constraint = signer_token_account.owner == signer.key() @ BridgeError::InvalidAmount,
        constraint = signer_token_account.mint == bridge_state.szec_mint @ BridgeError::InvalidAmount
    )]
    pub signer_token_account: Account<'info, TokenAccount>,

    /// The sZEC token mint.
    ///
    /// Supply will be decreased by the burn amount.
    /// Must match the mint stored in bridge_state.
    #[account(
        mut,
        constraint = szec_mint.key() == bridge_state.szec_mint @ BridgeError::InvalidAmount
    )]
    pub szec_mint: Account<'info, Mint>,

    /// Bridge state for mint validation.
    ///
    /// Read-only reference to verify the correct mint is being burned.
    #[account(
        seeds = [b"bridge-state"],
        bump = bridge_state.bump
    )]
    pub bridge_state: Account<'info, BridgeState>,

    /// Token program for burn operation.
    pub token_program: Program<'info, Token>,
}

/// Accounts for updating the entire validator set.
///
/// This is a threshold action that replaces the complete validator set with
/// a new one. Requires signatures from the current validators meeting the
/// current threshold. The account is reallocated to fit the new validator set.
///
/// # Accounts
/// - `payer`: Transaction and realloc fee payer
/// - `bridge_state`: Updated with new validator set, threshold, and total count
/// - `system_program`: Required for reallocation
///
/// # Reallocation
/// The bridge_state account is reallocated to accommodate the new number of
/// validators. The payer covers any additional rent required.
#[derive(Accounts)]
#[instruction(new_validators: Vec<Pubkey>, new_threshold: u16, signatures: Vec<[u8; 64]>)]
pub struct UpdateValidatorsFull<'info> {
    /// Transaction fee payer and reallocation payer.
    ///
    /// Covers the cost of resizing the bridge_state account.
    #[account(mut)]
    pub payer: Signer<'info>,

    /// Bridge state PDA storing validator set.
    ///
    /// Reallocated to fit the new validator set size.
    /// Updated with new validators, threshold, and total count.
    /// Nonce is incremented after update.
    #[account(
        mut,
        seeds = [b"bridge-state"],
        bump = bridge_state.bump,
        realloc = BridgeState::space(new_validators.len()),
        realloc::payer = payer,
        realloc::zero = false
    )]
    pub bridge_state: Account<'info, BridgeState>,

    /// System program for account reallocation.
    pub system_program: Program<'info, System>,

    /// Instructions sysvar for Ed25519 signature verification.
    ///
    /// Used to read Ed25519 program verification results.
    ///
    /// CHECK: Must be the Instructions sysvar account
    #[account(address = anchor_lang::solana_program::sysvar::instructions::ID)]
    pub instructions: UncheckedAccount<'info>,
}

/// Accounts for adding a single validator to the set.
///
/// This is a threshold action that adds one new validator to the existing set.
/// Requires signatures from the current validators meeting the threshold.
/// The account is reallocated to accommodate the additional validator.
///
/// # Accounts
/// - `payer`: Transaction and realloc fee payer
/// - `bridge_state`: Validator is appended, total count incremented
/// - `system_program`: Required for reallocation
///
/// # Reallocation
/// The bridge_state account grows by one validator's space (32 bytes).
#[derive(Accounts)]
#[instruction(validator: Pubkey, signatures: Vec<[u8; 64]>)]
pub struct AddValidator<'info> {
    /// Transaction fee payer and reallocation payer.
    ///
    /// Covers the cost of growing the bridge_state account.
    #[account(mut)]
    pub payer: Signer<'info>,

    /// Bridge state PDA storing validator set.
    ///
    /// Reallocated to fit one additional validator.
    /// New validator is appended to the validators vector.
    /// Total count is incremented, nonce is incremented.
    #[account(
        mut,
        seeds = [b"bridge-state"],
        bump = bridge_state.bump,
        realloc = BridgeState::space(bridge_state.validators.len() + 1),
        realloc::payer = payer,
        realloc::zero = false
    )]
    pub bridge_state: Account<'info, BridgeState>,

    /// System program for account reallocation.
    pub system_program: Program<'info, System>,

    /// Instructions sysvar for Ed25519 signature verification.
    ///
    /// Used to read Ed25519 program verification results.
    ///
    /// CHECK: Must be the Instructions sysvar account
    #[account(address = anchor_lang::solana_program::sysvar::instructions::ID)]
    pub instructions: UncheckedAccount<'info>,
}

/// Accounts for removing a single validator from the set.
///
/// This is a threshold action that removes one validator from the existing set.
/// Requires signatures from the current validators meeting the threshold.
/// The account is reallocated to the smaller size. Removal is only allowed if
/// the threshold can still be met after removal.
///
/// # Accounts
/// - `payer`: Transaction and realloc fee payer (may receive rent refund)
/// - `bridge_state`: Validator is removed, total count decremented
/// - `system_program`: Required for reallocation
///
/// # Reallocation
/// The bridge_state account shrinks by one validator's space (32 bytes).
/// The payer may receive a rent refund for the released space.
///
/// # Safety
/// The instruction validates that removal won't violate the threshold requirement
/// (i.e., threshold <= total_validators - 1).
#[derive(Accounts)]
#[instruction(validator: Pubkey, signatures: Vec<[u8; 64]>)]
pub struct RemoveValidator<'info> {
    /// Transaction fee payer and reallocation payer.
    ///
    /// May receive rent refund from shrinking the account.
    #[account(mut)]
    pub payer: Signer<'info>,

    /// Bridge state PDA storing validator set.
    ///
    /// Reallocated to fit one fewer validator.
    /// Specified validator is removed from the validators vector.
    /// Total count is decremented, nonce is incremented.
    #[account(
        mut,
        seeds = [b"bridge-state"],
        bump = bridge_state.bump,
        realloc = BridgeState::space(bridge_state.validators.len().saturating_sub(1)),
        realloc::payer = payer,
        realloc::zero = false
    )]
    pub bridge_state: Account<'info, BridgeState>,

    /// System program for account reallocation.
    pub system_program: Program<'info, System>,

    /// Instructions sysvar for Ed25519 signature verification.
    ///
    /// Used to read Ed25519 program verification results.
    ///
    /// CHECK: Must be the Instructions sysvar account
    #[account(address = anchor_lang::solana_program::sysvar::instructions::ID)]
    pub instructions: UncheckedAccount<'info>,
}
