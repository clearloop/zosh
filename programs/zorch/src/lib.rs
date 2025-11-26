//! zorch consensus program

use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};
pub use errors::BridgeError;
pub use events::{BurnEvent, MintEvent, ValidatorSetUpdated};
use handler::{external, internal, threshold};
pub use state::BridgeState;

declare_id!("2KwobV7wjmUzGRQfpd3G5HVRfCRUXfry9MoM3Hbks9dz");

pub mod api;
pub mod errors;
pub mod events;
mod handler;
pub mod state;
pub mod types;
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

    /// Update token metadata (internal action, authority only)
    pub fn metadata(
        ctx: Context<UpdateMetadata>,
        name: String,
        symbol: String,
        uri: String,
    ) -> Result<()> {
        internal::metadata(ctx, name, symbol, uri)
    }

    /// Mint sZEC to recipients (threshold action, supports batch)
    pub fn mint<'info>(
        ctx: Context<'_, '_, '_, 'info, MintZec<'info>>,
        mints: Vec<types::MintEntry>,
        signatures: Vec<[u8; 64]>,
    ) -> Result<()> {
        threshold::mint(ctx, mints, signatures)
    }

    /// Burn sZEC to bridge back to Zcash (public action)
    pub fn burn(ctx: Context<BurnZec>, amount: u64, zec_recipient: String) -> Result<()> {
        external::burn(ctx, amount, zec_recipient)
    }

    /// Update the entire validator set (threshold action)
    pub fn validators(
        ctx: Context<Validators>,
        new_validators: Vec<Pubkey>,
        new_threshold: u8,
        signatures: Vec<[u8; 64]>,
    ) -> Result<()> {
        threshold::validators(ctx, new_validators, new_threshold, signatures)
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
/// - `zec_mint`: The SPL token mint for sZEC with 8 decimals (matching ZEC)
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
    /// - Seeds `[b"zec-mint"]` for deterministic address
    #[account(
        init,
        payer = payer,
        mint::decimals = 8,
        mint::authority = bridge_state,
        seeds = [b"zec-mint"],
        bump
    )]
    pub zec_mint: Account<'info, Mint>,

    /// System program for account creation.
    pub system_program: Program<'info, System>,

    /// Token program for mint creation.
    pub token_program: Program<'info, Token>,

    /// Rent sysvar for rent calculations.
    pub rent: Sysvar<'info, Rent>,
}

/// Accounts for updating token metadata.
///
/// This is an internal action that can only be called by the bridge authority.
/// It creates or updates the Metaplex token metadata for the sZEC mint.
///
/// # Accounts
/// - `authority`: Bridge authority (must match bridge_state.authority)
/// - `bridge_state`: Read-only reference for authority validation
/// - `zec_mint`: The sZEC token mint
/// - `metadata`: Metaplex metadata account (PDA derived from mint)
/// - `token_metadata_program`: Metaplex Token Metadata program
/// - `system_program`: Required for account creation
/// - `rent`: Rent sysvar
#[derive(Accounts)]
pub struct UpdateMetadata<'info> {
    /// Bridge authority that can update metadata.
    ///
    /// Must match the authority stored in bridge_state.
    #[account(
        mut,
        constraint = authority.key() == bridge_state.authority @ BridgeError::InvalidRecipient
    )]
    pub authority: Signer<'info>,

    /// Bridge state for authority validation.
    #[account(
        seeds = [b"bridge-state"],
        bump = bridge_state.bump
    )]
    pub bridge_state: Account<'info, BridgeState>,

    /// The sZEC token mint.
    #[account(
        mut,
        seeds = [b"zec-mint"],
        bump,
        constraint = zec_mint.key() == bridge_state.zec_mint @ BridgeError::InvalidMint
    )]
    pub zec_mint: Account<'info, Mint>,

    /// Metaplex metadata account for the mint.
    ///
    /// This is a PDA derived from the mint address.
    /// Will be created if it doesn't exist, or updated if it does.
    ///
    /// CHECK: Validated by Metaplex program
    #[account(mut)]
    pub metadata: UncheckedAccount<'info>,

    /// Metaplex Token Metadata program.
    ///
    /// CHECK: Validated by constraint
    #[account(address = mpl_token_metadata::ID)]
    pub token_metadata_program: UncheckedAccount<'info>,

    /// System program for account creation.
    pub system_program: Program<'info, System>,

    /// Rent sysvar.
    pub rent: Sysvar<'info, Rent>,
}

/// Accounts for minting sZEC tokens.
///
/// This is a threshold action that requires signatures from validators meeting
/// the threshold requirement. Validators sign off-chain and provide signatures
/// to authorize the mint operation.
///
/// Supports both single and batch minting. For batch minting, pass multiple
/// recipient token accounts via remaining_accounts in the same order as the
/// mints vector.
///
/// # Accounts
/// - `payer`: Transaction fee payer
/// - `bridge_state`: Stores validator set and is used as mint authority
/// - `zec_mint`: The sZEC token mint
/// - `token_program`: Required for minting
/// - `system_program`: Required for various operations
/// - `instructions`: Instructions sysvar for signature verification
///
/// # Remaining Accounts
/// - One token account per mint in the mints vector
/// - Each must be for the sZEC mint and owned by the corresponding recipient
#[derive(Accounts)]
#[instruction(mints: Vec<types::MintEntry>, signatures: Vec<[u8; 64]>)]
pub struct MintZec<'info> {
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
        seeds = [b"zec-mint"],
        bump,
        constraint = zec_mint.key() == bridge_state.zec_mint @ BridgeError::InvalidMint
    )]
    pub zec_mint: Account<'info, Mint>,

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
/// - `zec_mint`: The sZEC token mint (supply will decrease)
/// - `bridge_state`: Read-only reference for mint validation
/// - `token_program`: Required for burn operation
///
/// # Constraints
/// - Signer must own the token account
/// - Token account must hold sZEC tokens
/// - Mint must match bridge state's recorded mint
#[derive(Accounts)]
pub struct BurnZec<'info> {
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
        constraint = signer_token_account.mint == bridge_state.zec_mint @ BridgeError::InvalidAmount
    )]
    pub signer_token_account: Account<'info, TokenAccount>,

    /// The sZEC token mint.
    ///
    /// Supply will be decreased by the burn amount.
    /// Must match the mint stored in bridge_state.
    #[account(
        mut,
        constraint = zec_mint.key() == bridge_state.zec_mint @ BridgeError::InvalidAmount
    )]
    pub zec_mint: Account<'info, Mint>,

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
pub struct Validators<'info> {
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
