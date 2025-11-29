//! The import interfaces for the runtime

use crate::{Config, Runtime, Storage};
use anyhow::Result;
use zcore::Block;

impl<C: Config> Runtime<C> {
    /// Import a new block
    ///
    /// On success, at the node side we need to subscribe to other nodes.
    pub async fn import(&mut self, block: Block) -> Result<()> {
        // TODO: do the verifications of the block

        // set block to the storage
        self.storage.set_block(block)?;
        Ok(())
    }
}
