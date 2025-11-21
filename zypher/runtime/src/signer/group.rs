//! Zcash group signers

use anyhow::Result;
use reddsa::frost::redjubjub::{
    self,
    keys::{self, KeyPackage, PublicKeyPackage, SecretShare},
    round1, round2, Identifier, RandomizedParams, Randomizer, Signature, SigningPackage,
    VerifyingKey,
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
    /// Returns the signature and the randomized verifying key (needed for verification)
    pub fn sign_message(&self, message: &[u8]) -> Result<(Signature, VerifyingKey)> {
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
        let randomizer = Randomizer::new(rand_core::OsRng, &signing_package)?;
        for (identifier, nonce) in nonces {
            let keypkg = &keypkgs[identifier];
            let share = round2::sign(&signing_package, &nonce, keypkg, randomizer)?;
            signatures.insert(*identifier, share);
        }

        // aggregate the signature shares
        let params = RandomizedParams::from_randomizer(self.package.verifying_key(), randomizer);
        let signature =
            redjubjub::aggregate(&signing_package, &signatures, &self.package, &params)?;
        Ok((signature, *params.randomized_verifying_key()))
    }
}

#[test]
fn test_redjubjub_aggregate() {
    let group = ZcashGroupSigners::new(3, 2).unwrap();
    let message = b"zypherpunk";
    let (signature, verifying_key) = group.sign_message(message).unwrap();
    // For rerandomized FROST, signatures must be verified with the randomized verifying key
    // (not the original verifying key)
    assert!(verifying_key.verify(message, &signature).is_ok())
}
