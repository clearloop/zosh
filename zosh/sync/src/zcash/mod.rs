//! Zcash related stuffs for zorch

use orchard::circuit::ProvingKey;
use std::sync::LazyLock;
use zcash_client_backend::data_api::wallet::ConfirmationsPolicy;
pub use {
    cmd::Zcash,
    light::{Config, ZcashClient},
    orchard::Address,
    signer::{GroupSigners, ShareSigner, SignerInfo},
    zcash_keys::{address::UnifiedAddress, encoding::AddressCodec, keys::UnifiedFullViewingKey},
    zcash_primitives::consensus::Network,
    zcash_protocol::consensus::BranchId,
};

mod cmd;
mod light;
mod signer;

/// The confirmations policy for the zcash light client
pub static CONFIRMATIONS: ConfirmationsPolicy = ConfirmationsPolicy::MIN;

/// The proving key for the orchard transactions
pub static PROVING_KEY: LazyLock<ProvingKey> = LazyLock::new(ProvingKey::build);
