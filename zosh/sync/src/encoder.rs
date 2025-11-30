//! Address encoder for the zorch bridge

use crate::{
    solana::{Pubkey, Signature},
    zcash::{AddressCodec, Network, TxId, UnifiedAddress},
};
use anyhow::{Context, Result};
use zcore::FixedBytes;

/// The address encoder for zcash and solana addresses
pub trait ChainFormatEncoder {
    /// Encode the solana address to the chain format
    fn solana_address(&self) -> Result<Pubkey>;

    /// Encode the zcash address to the chain format
    fn zcash_address(&self, network: &Network) -> Result<UnifiedAddress>;

    /// Encode the solana signature to the chain format
    fn solana_signature(&self) -> Result<Signature>;

    /// Encode the zcash signature to the chain format
    fn zcash_signature(&self) -> Result<TxId>;
}

impl<T: AsRef<[u8]>> ChainFormatEncoder for T {
    fn solana_address(&self) -> Result<Pubkey> {
        Ok(Pubkey::new_from_array(
            self.bytes32().context("Invalid solana address")?,
        ))
    }

    fn zcash_address(&self, network: &Network) -> Result<UnifiedAddress> {
        UnifiedAddress::decode(network, String::from_utf8(self.as_ref().to_vec())?.as_str())
            .map_err(|e| anyhow::anyhow!("Invalid zcash address: {e:?}"))
    }

    fn solana_signature(&self) -> Result<Signature> {
        Ok(Signature::from(
            self.bytes64().context("Invalid solana signature")?,
        ))
    }

    fn zcash_signature(&self) -> Result<TxId> {
        Ok(TxId::from_bytes(
            self.bytes32().context("Invalid zcash signature")?,
        ))
    }
}
