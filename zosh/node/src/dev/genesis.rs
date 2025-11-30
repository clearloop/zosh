//! The development genesis block

use anyhow::Result;
use runtime::storage::Commit;
use solana_signer::Signer;
use sync::solana::dev;
use zcore::{bft::Bft, state::key, Head, State};

/// The genesis state for the development node
pub fn commit() -> Result<Commit> {
    let authority = dev::load_authority()?;
    let ident = authority.pubkey().to_bytes();
    let mut commit = Commit::default();
    let state = State::default();
    let head = Head {
        slot: 0,
        hash: [0; 32],
    };
    let bft = Bft {
        validators: vec![ident],
        threshold: 1,
        series: vec![],
    };

    commit.insert(key::ACCUMULATOR_KEY, state.accumulator.to_vec());
    commit.insert(key::BFT_KEY, postcard::to_allocvec(&bft)?);
    commit.insert(key::PRESENT_KEY, postcard::to_allocvec(&head)?);
    Ok(commit)
}
