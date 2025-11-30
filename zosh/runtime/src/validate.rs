//! The validation interfaces for the runtime

use crate::{Config, Runtime, Storage};
use anyhow::Result;
use zcore::{Block, Ed25519Signature, Extrinsic};

impl<C: Config> Runtime<C> {
    /// Validate the block, this happens on the network layer for yielding
    /// the current node's vote of the block.
    pub async fn validate(&mut self, block: Block) -> Result<Ed25519Signature> {
        self.validate_duplications(&block.extrinsic)?;
        self.sync.validate_bridges(&block.extrinsic.bridge)?;
        self.sync.validate_receipts(&block.extrinsic.receipts)?;
        let _hash = block.header.hash();

        // TODO: sign the hash with validators' keypair
        todo!()
    }

    /// Validate the duplications of the extrinsic
    fn validate_duplications(&self, ex: &Extrinsic) -> Result<()> {
        let txs = ex.transactions();
        for tx in txs {
            if self.storage.exists(&tx) {
                anyhow::bail!("Transaction already processed");
            }
        }
        Ok(())
    }
}
