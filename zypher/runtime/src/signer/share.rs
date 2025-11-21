//! Share of a ZypherBridge client

use anyhow::Result;
use orchard::keys::{FullViewingKey, SpendValidatingKey, SpendingKey};
use rand::Rng;
use reddsa::frost::redjubjub::{
    keys::{PublicKeyPackage, SecretShare},
    Identifier,
};

/// Zcash signer of a ZypherBridge client
#[derive(Debug)]
pub struct ZcashSharedSigner {
    /// frost redjubjub identifier
    pub identifier: Identifier,

    /// frost redjubjub public key package
    pub rjpackage: PublicKeyPackage,

    /// frost redjubjub secret share
    pub rjshare: SecretShare,
}

impl ZcashSharedSigner {
    /// Get the orchard full viewing key
    pub fn orchard(&self) -> Result<FullViewingKey> {
        let ak = self.rjpackage.verifying_key().serialize()?;
        let ak = SpendValidatingKey::from_bytes(&ak)
            .ok_or(anyhow::anyhow!("Invalid spend validating key"))?;

        let mut rng = rand::rng();
        let sk = loop {
            let random_bytes = rng.random::<[u8; 32]>();
            let sk = SpendingKey::from_bytes(random_bytes);
            if let Some(sk) = sk.into_option() {
                break sk;
            }
        };

        Ok(FullViewingKey::from_sk_and_ak(&sk, ak))
    }
}
