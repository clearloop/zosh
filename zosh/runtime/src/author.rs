//! The author interfaces for the runtime

use crate::{Config, Runtime, Storage};
use anyhow::Result;
use zcore::{Block, Header};

impl<C: Config> Runtime<C> {
    /// Author an unauthorized block
    pub async fn author(&mut self) -> Result<Block> {
        let state = self.storage.state();
        let parent = state.present;

        // get the extrinsic from the pool
        let rawex = postcard::to_allocvec(&self.pool.extrinsic)?;
        let extrinsic = crypto::blake3(&rawex);

        // Build the header first
        let root = self.storage.root();
        let header = Header {
            height: parent.height + 1,
            parent: parent.hash,
            state: root,
            extrinsic,
            votes: Default::default(),
        };

        // clean up of the mempool
        let extrinsic = self.pool.extrinsic.clone();
        self.pool.reset_extrinsic();
        Ok(Block { header, extrinsic })
    }
}
