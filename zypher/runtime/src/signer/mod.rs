//! Signer of a ZypherBridge client

use crate::config;
use anyhow::Result;
use reddsa::frost::redjubjub::Identifier;
pub use solana_sdk::signer::keypair::Keypair;
pub use {group::ZcashGroupSigners, share::ZcashSharedSigner};

mod group;
mod share;

/// Signer of a ZypherBridge client
#[derive(Debug)]
pub struct Signer {
    /// zcash signer
    pub zcash: Option<ZcashSharedSigner>,

    /// solana keypair
    pub solana: Keypair,
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
            zcash: Some(ZcashSharedSigner {
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
