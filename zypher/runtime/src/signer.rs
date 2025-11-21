//! Signer of a ZypherBridge client

use crate::config;
use anyhow::Result;
use orchard::keys::{FullViewingKey, SpendValidatingKey, SpendingKey};
use rand::Rng;
use reddsa::frost::redjubjub::{
    keys::{PublicKeyPackage, SecretShare},
    Identifier,
};
pub use solana_sdk::signer::keypair::Keypair;

/// Signer of a ZypherBridge client
#[derive(Debug)]
pub struct Signer {
    /// zcash signer
    pub zcash: Option<ZcashSigner>,

    /// solana keypair
    pub solana: Keypair,
}

/// Zcash signer of a ZypherBridge client
#[derive(Debug)]
pub struct ZcashSigner {
    /// frost redjubjub identifier
    pub identifier: Identifier,

    /// frost redjubjub public key package
    pub rjpackage: PublicKeyPackage,

    /// frost redjubjub secret share
    pub rjshare: SecretShare,
}

impl ZcashSigner {
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

impl TryFrom<&config::Key> for Signer {
    type Error = anyhow::Error;

    fn try_from(key: &config::Key) -> Result<Self> {
        let Some(zcash) = &key.zcash else {
            return Ok(Signer {
                zcash: None,
                solana: Keypair::from_base58_string(&key.solana),
            });
        };

        let ident = bs58::decode(&zcash.identifier).into_vec()?;
        let rjpkg = bs58::decode(&zcash.package).into_vec()?;
        let rjshare = bs58::decode(&zcash.share).into_vec()?;
        Ok(Signer {
            zcash: Some(ZcashSigner {
                identifier: Identifier::deserialize(&ident)?,
                rjpackage: postcard::from_bytes(&rjpkg)?,
                rjshare: postcard::from_bytes(&rjshare)?,
            }),
            solana: Keypair::from_base58_string(&key.solana),
        })
    }
}

impl TryFrom<Signer> for config::Key {
    type Error = anyhow::Error;

    fn try_from(signer: Signer) -> Result<Self> {
        Ok(config::Key {
            zcash: if let Some(zcash) = &signer.zcash {
                Some(config::Frost {
                    identifier: bs58::encode(&postcard::to_allocvec(&zcash.identifier)?)
                        .into_string(),
                    package: bs58::encode(&postcard::to_allocvec(&zcash.rjpackage)?).into_string(),
                    share: bs58::encode(&postcard::to_allocvec(&zcash.rjshare)?).into_string(),
                })
            } else {
                None
            },
            solana: signer.solana.to_base58_string(),
        })
    }
}
