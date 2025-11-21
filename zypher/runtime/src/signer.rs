//! Signer of a ZypherBridge client

use crate::config;
use anyhow::Result;
use reddsa::frost::redjubjub::{
    keys::{PublicKeyPackage, SecretShare},
    Identifier,
};
pub use solana_sdk::signer::keypair::Keypair;

/// Signer of a ZypherBridge client
#[derive(Debug)]
pub struct Signer {
    /// frost redjubjub identifier
    pub identifier: Identifier,

    /// frost redjubjub public key package
    pub rjpackage: PublicKeyPackage,

    /// frost redjubjub secret share
    pub rjshare: SecretShare,

    /// solana keypair
    pub solana: Keypair,
}

impl TryFrom<&config::Key> for Signer {
    type Error = anyhow::Error;

    fn try_from(key: &config::Key) -> Result<Self> {
        let ident = bs58::decode(&key.zcash.identifier).into_vec()?;
        let rjpkg = bs58::decode(&key.zcash.package).into_vec()?;
        let rjshare = bs58::decode(&key.zcash.share).into_vec()?;
        Ok(Signer {
            identifier: Identifier::deserialize(&ident)?,
            rjpackage: postcard::from_bytes(&rjpkg)?,
            rjshare: postcard::from_bytes(&rjshare)?,
            solana: Keypair::from_base58_string(&key.solana),
        })
    }
}

impl TryFrom<Signer> for config::Key {
    type Error = anyhow::Error;

    fn try_from(signer: Signer) -> Result<Self> {
        Ok(config::Key {
            zcash: config::Frost {
                identifier: bs58::encode(&postcard::to_allocvec(&signer.identifier)?).into_string(),
                package: bs58::encode(&postcard::to_allocvec(&signer.rjpackage)?).into_string(),
                share: bs58::encode(&postcard::to_allocvec(&signer.rjshare)?).into_string(),
            },
            solana: signer.solana.to_base58_string(),
        })
    }
}
