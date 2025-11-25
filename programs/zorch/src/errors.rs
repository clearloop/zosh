use anchor_lang::prelude::*;

#[error_code]
pub enum BridgeError {
    #[msg("Invalid threshold: threshold must be less than or equal to total validators")]
    InvalidThreshold,

    #[msg("Insufficient signatures: threshold not met")]
    InsufficientSignatures,

    #[msg("Invalid signature provided")]
    InvalidSignature,

    #[msg("Signer not in validator set")]
    SignerNotValidator,

    #[msg("Duplicate signer detected")]
    DuplicateSigner,

    #[msg("Action already executed")]
    ActionAlreadyExecuted,

    #[msg("Invalid amount: must be greater than zero")]
    InvalidAmount,

    #[msg("Invalid mint: mint must be for the sZEC token")]
    InvalidMint,

    #[msg("Invalid recipient: recipient must be the owner of the token account")]
    InvalidRecipient,

    #[msg("Validator already exists in the set")]
    ValidatorAlreadyExists,

    #[msg("Validator not found in the set")]
    ValidatorNotFound,

    #[msg("Cannot remove validator: would violate threshold requirement")]
    CannotRemoveValidator,

    #[msg("Invalid Zcash address format")]
    InvalidZcashAddress,

    #[msg("Maximum validators limit reached")]
    MaxValidatorsReached,
}
