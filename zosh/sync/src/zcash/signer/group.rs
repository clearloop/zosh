//! Zcash group signers

use anyhow::Result;
use orchard::{
    builder::MaybeSigned,
    circuit::ProvingKey,
    keys::SpendValidatingKey,
    primitives::redpallas::{Signature, SpendAuth},
};
use reddsa::frost::redpallas::{
    self,
    keys::{self, KeyPackage, PublicKeyPackage, SecretShare},
    round1, round2, Identifier, RandomizedParams, Randomizer, SigningPackage, VerifyingKey,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use zcash_primitives::transaction::{
    sighash::{signature_hash, SignableInput},
    txid::TxIdDigester,
    Authorized, TransactionData, Unauthorized,
};

use crate::zcash::SignerInfo;

/// Zcash group signers
#[derive(Serialize, Deserialize)]
pub struct GroupSigners {
    /// shares of the signers
    pub shares: BTreeMap<Identifier, SecretShare>,

    /// public key package of the signers
    pub package: PublicKeyPackage,

    /// minimum number of signers required to sign a message
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
    /// Returns the signature and the randomized verifying key (needed for verification)
    pub fn sign(
        &self,
        message: &[u8],
        randomizer: &Randomizer,
    ) -> Result<(redpallas::Signature, VerifyingKey)> {
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
            let share = round2::sign(&signing_package, &nonce, keypkg, *randomizer)?;
            signatures.insert(*identifier, share);
        }

        // aggregate the signature shares
        let params = RandomizedParams::from_randomizer(self.package.verifying_key(), *randomizer);
        let signature =
            redpallas::aggregate(&signing_package, &signatures, &self.package, &params)?;
        Ok((signature, *params.randomized_verifying_key()))
    }

    /// Sign a transaction with the group of signers
    pub fn sign_tx(
        &self,
        utx: TransactionData<Unauthorized>,
    ) -> Result<TransactionData<Authorized>> {
        let fvk = self.orchard()?;
        let txid_parts = utx.digest(TxIdDigester);
        let sighash = signature_hash(&utx, &SignableInput::Shielded, &txid_parts);
        let proving_key = ProvingKey::build();
        let proven = utx
            .orchard_bundle()
            .cloned()
            .ok_or(anyhow::anyhow!("Failed to get orchard bundle"))?
            .create_proof(&proving_key, rand_core::OsRng)?
            .prepare(rand_core::OsRng, *sighash.as_ref());
        let ak: SpendValidatingKey = fvk.into();
        let mut alphas = Vec::new();
        let proven = proven.map_authorization(
            &mut rand_core::OsRng,
            |_rng, _partial, maybe| {
                if let MaybeSigned::SigningMetadata(parts) = &maybe {
                    if parts.ak == ak {
                        alphas.push(parts.alpha);
                    }
                }
                maybe
            },
            |_rng, auth| auth,
        );

        // Sign the transaction
        let mut signatures = Vec::new();
        for alpha in alphas.iter() {
            let randomizer = Randomizer::from_scalar(*alpha);
            let (signature, _) = self.sign(sighash.as_ref(), &randomizer)?;
            let sigbytes: [u8; 64] = signature
                .serialize()?
                .try_into()
                .map_err(|_e| anyhow::anyhow!("Failed to convert signature to bytes"))?;
            let signature = Signature::<SpendAuth>::from(sigbytes);
            signatures.push(signature);
        }

        let proven = proven
            .append_signatures(&signatures)
            .map_err(|_e| anyhow::anyhow!("Failed to append signatures"))?
            .finalize()
            .map_err(|_e| anyhow::anyhow!("Failed to finalize"))?;

        Ok(TransactionData::<Authorized>::from_parts(
            utx.version(),
            utx.consensus_branch_id(),
            0,
            utx.expiry_height(),
            None,
            None,
            None,
            Some(proven),
        ))
    }
}
