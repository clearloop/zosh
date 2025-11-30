//! The author interfaces for the runtime

use crate::{Config, Runtime, Storage};
use anyhow::Result;
use zcore::{Block, Hash, Header};

impl<C: Config> Runtime<C> {
    /// Author an unauthorized block
    pub async fn author(&mut self) -> Result<Block> {
        let state = self.storage.state();
        let parent = state.present;

        // get the extrinsic from the pool
        let rawex = postcard::to_allocvec(&self.pool.extrinsic)?;
        let extrinsic = crypto::blake3(&rawex);

        // Build the header first
        let root = self.storage.root()?;
        let txs = self.pool.extrinsic.txs();
        let accumulator = self.accumulate(parent.hash, txs)?;
        let header = Header {
            // TODO: if the previous lead failed to author lock, we need to
            // skip the slot and use the next slot.
            slot: parent.slot + 1,
            parent: parent.hash,
            state: root,
            accumulator,
            extrinsic,
            votes: Default::default(),
        };

        // clean up of the mempool
        let extrinsic = self.pool.extrinsic.clone();
        self.pool.reset_extrinsic();
        Ok(Block { header, extrinsic })
    }

    /// Accumulate the signatures of the extrinsic
    pub fn accumulate(&self, prev: Hash, txs: Vec<Vec<u8>>) -> Result<[u8; 32]> {
        let mut accumulator = prev.to_vec();
        accumulator.extend_from_slice(&txs.into_iter().flatten().collect::<Vec<u8>>());
        Ok(crypto::blake3(&accumulator))
    }
}
