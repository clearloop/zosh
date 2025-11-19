//! signer interface for the bridge

use anyhow::Result;
use rand::Rng;
use zcash_keys::keys::{UnifiedFullViewingKey, UnifiedIncomingViewingKey, UnifiedSpendingKey};
use zcash_primitives::{consensus::MainNetwork, zip32::AccountId};

/// Signer interface for the bridge
pub struct Signer {
    /// A ZIP 316 unified full viewing key.
    pub full_viewing: UnifiedFullViewingKey,
    /// A ZIP 316 unified incoming viewing key.
    pub incoming_viewing: UnifiedIncomingViewingKey,
    /// A set of spending keys that are all associated with a single ZIP-0032 account identifier.
    pub spending: UnifiedSpendingKey,
}

impl Signer {
    /// Create a new signer from a seed
    pub fn new(seed: &[u8]) -> Result<Self> {
        let spending = UnifiedSpendingKey::from_seed(&MainNetwork, seed, AccountId::ZERO)?;
        let viewing = spending.to_unified_full_viewing_key();
        let incoming = viewing.to_unified_incoming_viewing_key();
        Ok(Self {
            full_viewing: viewing,
            incoming_viewing: incoming,
            spending,
        })
    }

    /// Create a new signer from a random seed
    pub fn random() -> Result<Self> {
        let mut seed = [0u8; 32];
        rand::rng().fill(&mut seed);
        Self::new(&seed)
    }
}
