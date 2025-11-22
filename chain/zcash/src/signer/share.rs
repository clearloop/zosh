//! Share of a ZypherBridge client

use anyhow::Result;
use orchard::{
    keys::{FullViewingKey, Scope, SpendValidatingKey, SpendingKey},
    Address,
};
use reddsa::frost::redpallas::{
    keys::{PublicKeyPackage, SecretShare},
    Identifier,
};
use zcash_keys::{address::UnifiedAddress, keys::UnifiedFullViewingKey};

/// Zcash signer of a ZypherBridge client
#[derive(Debug)]
pub struct ShareSigner {
    /// frost redjubjub identifier
    pub identifier: Identifier,

    /// frost redjubjub public key package
    pub rjpackage: PublicKeyPackage,

    /// frost redjubjub secret share
    pub rjshare: SecretShare,
}

impl ShareSigner {
    /// Get the orchard full viewing key
    pub fn orchard(&self) -> Result<FullViewingKey> {
        let ak = self.rjpackage.verifying_key().serialize()?;
        let ak = SpendValidatingKey::from_bytes(&ak)
            .ok_or(anyhow::anyhow!("Invalid spend validating key"))?;
        let sk = SpendingKey::from_bytes([0; 32]).unwrap();
        Ok(FullViewingKey::from_sk_and_ak(&sk, ak))
    }

    /// Get the external address of the share
    pub fn external_address(&self) -> Result<Address> {
        let fvk = self.orchard()?;
        let address = fvk.address_at(0u64, Scope::External);
        Ok(address)
    }

    /// Get the unified address of the share
    pub fn unified_address(&self) -> Result<UnifiedAddress> {
        let addr = self.external_address()?;
        let uaddr = UnifiedAddress::from_receivers(Some(addr), None, None)
            .ok_or(anyhow::anyhow!("Invalid unified address"))?;
        Ok(uaddr)
    }

    /// Get the unified full viewing key of the share
    pub fn ufvk(&self) -> Result<UnifiedFullViewingKey> {
        let fvk = self.orchard()?;
        let ufvk = UnifiedFullViewingKey::from_orchard_fvk(fvk)?;
        Ok(ufvk)
    }
}
