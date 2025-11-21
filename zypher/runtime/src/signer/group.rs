//! Zcash group signers

use anyhow::Result;
use reddsa::frost::redjubjub::{
    self,
    keys::{self, KeyPackage, PublicKeyPackage, SecretShare},
    round1, round2, Identifier, RandomizedParams, Randomizer, Signature, SigningPackage,
};
use std::collections::BTreeMap;

/// Zcash group signers
pub struct ZcashGroupSigners {
    /// shares of the signers
    pub shares: BTreeMap<Identifier, SecretShare>,

    /// public key package of the signers
    pub package: PublicKeyPackage,

    /// minimum number of signers required to sign a message
    pub min: u16,
}

impl ZcashGroupSigners {
    /// Generate a new group of signers
    pub fn new(max: u16, min: u16) -> Result<Self> {
        let mut rng = rand_core::OsRng;
        let (shares, package) =
            keys::generate_with_dealer(max, min, keys::IdentifierList::Default, &mut rng)?;
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
        let randomizer = Randomizer::new(&mut rand_core::OsRng, &signing_package)?;
        for (identifier, nonce) in nonces {
            let keypkg = &keypkgs[identifier];
            let share = round2::sign(&signing_package, &nonce, &keypkg, randomizer)?;
            signatures.insert(*identifier, share);
        }

        // returns the signature
        let params = RandomizedParams::new(
            &self.package.verifying_key(),
            &signing_package,
            &mut rand_core::OsRng,
        )?;

        redjubjub::aggregate(&signing_package, &signatures, &self.package, &params)
            .map_err(Into::into)
    }
}

#[test]
fn test_redjubjub_aggregate() {
    let group = ZcashGroupSigners::new(3, 2).unwrap();
    let signature = group.sign(b"zypherpunk").unwrap();
    let verifying_key = group.package.verifying_key();
    assert!(verifying_key.verify(b"zypherpunk", &signature).is_ok())
}
