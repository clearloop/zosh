//! The import interfaces for the runtime

use crate::{Config, Runtime, Storage};
use anyhow::Result;
use zcore::{Block, Ed25519Signature};

impl<C: Config> Runtime<C> {
    /// Import a new block
    ///
    /// On success, at the node side we need to subscribe to other nodes.
    ///
    /// NOTE: the validation happens on the network layer, at the case
    /// the QC is satified, we can import the block directly.
    pub fn import(&mut self, block: Block) -> Result<()> {
        let state = self.storage.state();
        state.bft.validate_votes(&block.header)?;
        self.storage.set_block(block)?;
        Ok(())
    }

    /// Validate the block
    ///
    /// After the validation we return the signature from our side, this
    /// will be used for complete the QC of the block.
    pub async fn validate(&mut self, block: Block) -> Result<Ed25519Signature> {
        let hash = block.header.hash();

        self.sync
            .solana
            .sign_message(&hash)
            .map_err(Into::into)
            .map(|sig| *sig.as_array())
    }
}
