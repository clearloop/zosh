//! Group signers for the solana bridge
//!
//! This is only for development usages

use anyhow::Result;
use frost_ed25519::{
    keys::{self, KeyPackage, PublicKeyPackage, SecretShare},
    round1, round2, Identifier, Signature, SigningPackage,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// DEV_ONLY: Group signers for the solana bridge
#[derive(Debug, Serialize, Deserialize)]
pub struct GroupSigners {
    /// The shares of the signers
    pub shares: BTreeMap<Identifier, SecretShare>,

    /// The public key package of the signers
    pub package: PublicKeyPackage,

    /// The minimum number of signers required to sign a message
    pub min: u16,
}

impl GroupSigners {
    /// Generate a new group of signers
    pub fn new(max: u16, min: u16) -> Result<Self> {
        let rng = rand_core::OsRng;
        let (shares, package) =
            keys::generate_with_dealer(max, min, keys::IdentifierList::Default, rng)?;
        Ok(Self {
            shares,
            package,
            min,
        })
    }

    /// Sign a message with the group of signers
    pub fn sign(&self, message: &[u8]) -> Result<Signature> {
        let mut nonces = BTreeMap::new();
        let mut commitments = BTreeMap::new();
        let mut keypkgs = BTreeMap::new();
        for (identifier, share) in &self.shares {
            keypkgs.insert(identifier, KeyPackage::try_from(share.clone())?);
        }

        ////////////////////////////////////////////////////////////////////////////
        // Round 1: generating nonces and signing commitments for each participant
        ////////////////////////////////////////////////////////////////////////////
        for (identifier, keypkg) in keypkgs.iter() {
            let (nonce, commitment) = round1::commit(keypkg.signing_share(), &mut rand_core::OsRng);
            nonces.insert(*identifier, nonce);
            commitments.insert(**identifier, commitment);
        }

        ////////////////////////////////////////////////////////////////////////////
        // Round 2: each participant generates their signature share
        ////////////////////////////////////////////////////////////////////////////
        let mut signatures = BTreeMap::new();
        let signing_package = SigningPackage::new(commitments, message);
        for (identifier, nonce) in nonces {
            let keypkg = &keypkgs[identifier];
            let share = round2::sign(&signing_package, &nonce, keypkg)?;
            signatures.insert(*identifier, share);
        }

        // aggregate the signature shares
        frost_ed25519::aggregate(&signing_package, &signatures, &self.package).map_err(Into::into)
    }
}

#[test]
fn test_ed25519_group_signers() -> Result<()> {
    let group = GroupSigners::new(3, 2)?;
    let message = b"solana";
    let signature = group.sign(message)?;
    let verifying_key = group.package.verifying_key();
    verifying_key.verify(message, &signature)?;
    Ok(())
}
