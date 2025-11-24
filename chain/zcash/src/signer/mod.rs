//! Signers of zcash

use anyhow::Result;
use orchard::{
    keys::{FullViewingKey, Scope, SpendValidatingKey, SpendingKey},
    Address,
};
use reddsa::frost::redpallas::VerifyingKey;
use zcash_keys::{address::UnifiedAddress, keys::UnifiedFullViewingKey};
pub use {group::GroupSigners, share::ShareSigner};

mod group;
mod share;

/// Trait for signer information
pub trait SignerInfo {
    /// Get the verifying key
    fn verifying_key(&self) -> &VerifyingKey;

    /// Get the orchard full viewing key
    fn orchard(&self) -> Result<FullViewingKey> {
        let ak = self.verifying_key().serialize()?;
        let ak = SpendValidatingKey::from_bytes(&ak)
            .ok_or(anyhow::anyhow!("Invalid spend validating key"))?;
        let sk = SpendingKey::from_bytes([0; 32]).unwrap();
        Ok(FullViewingKey::from_sk_ak(&sk, ak))
    }

    /// Get the external address of the share
    fn external_address(&self) -> Result<Address> {
        let fvk = self.orchard()?;
        let address = fvk.address_at(0u64, Scope::External);
        Ok(address)
    }

    /// Get the unified address of the share
    fn unified_address(&self) -> Result<UnifiedAddress> {
        let addr = self.external_address()?;
        let uaddr = UnifiedAddress::from_receivers(Some(addr), None, None)
            .ok_or(anyhow::anyhow!("Invalid unified address"))?;
        Ok(uaddr)
    }

    /// Get the unified full viewing key of the share
    fn ufvk(&self) -> Result<UnifiedFullViewingKey> {
        let fvk = self.orchard()?;
        let ufvk = UnifiedFullViewingKey::from_orchard_fvk(fvk)?;
        Ok(ufvk)
    }
}

impl SignerInfo for ShareSigner {
    fn verifying_key(&self) -> &VerifyingKey {
        self.rjpackage.verifying_key()
    }
}

impl SignerInfo for GroupSigners {
    fn verifying_key(&self) -> &VerifyingKey {
        self.package.verifying_key()
    }
}
