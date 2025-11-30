//! The author interfaces for the runtime

use crate::{Config, Runtime, Storage};
use anyhow::Result;
use crypto::merkle;
use zcore::{Block, Hash, Header};

impl<C: Config> Runtime<C> {
    /// Author an unauthorized block
    pub async fn author(&mut self) -> Result<Block> {
        let state = self.storage.state();
        let parent = state.present;

        // get the extrinsic from the pool
        let extrinsic = self.pool.lock().await.pack()?;
        let txs = extrinsic.txs();
        let accumulator = self.accumulate(parent.hash, &txs)?;
        let root = merkle::root(txs);

        // Build the header first
        let header = Header {
            // TODO: if the previous lead failed to author lock, we need to
            // skip the slot and use the next slot.
            slot: parent.slot + 1,
            parent: parent.hash,
            state: root,
            accumulator,
            extrinsic: root,
            votes: Default::default(),
        };

        Ok(Block { header, extrinsic })
    }

    /// Accumulate the signatures of the extrinsic
    pub fn accumulate(&self, prev: Hash, txs: &Vec<Vec<u8>>) -> Result<[u8; 32]> {
        let mut accumulator = prev.to_vec();
        accumulator.extend_from_slice(&txs.iter().flatten().copied().collect::<Vec<u8>>());
        Ok(crypto::blake3(&accumulator))
    }
}
