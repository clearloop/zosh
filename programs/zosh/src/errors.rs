use anchor_lang::prelude::*;

#[error_code]
pub enum BridgeError {
    #[msg("Invalid amount: must be greater than zero")]
    InvalidAmount,

    #[msg("Invalid mint: mint must be for the sZEC token")]
    InvalidMint,

    #[msg("Invalid recipient: recipient must be the owner of the token account")]
    InvalidRecipient,

    #[msg("Invalid Zcash address format")]
    InvalidZcashAddress,

    #[msg("Invalid batch size: batch must contain at least 1 mint and not exceed the maximum")]
    InvalidBatchSize,

    #[msg("Invalid MPC signer: signer must be the MPC")]
    InvalidMpcSigner,
}
