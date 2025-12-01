//! The import interfaces for the runtime

use crate::{storage::Commit, Config, Runtime, Storage};
use anyhow::Result;
use zcore::{state::key, Block};

impl<C: Config> Runtime<C> {
    /// Import a new block
    ///
    /// On success, at the node side we need to subscribe to other nodes.
    ///
    /// NOTE: the validation happens on the network layer, at the case
    /// the QC is satisfied, we can import the block directly.
    ///
    /// TODO: but we need to validate the rotation of the validators here.
    pub fn import(&mut self, block: Block) -> Result<()> {
        let state = self.storage.state()?;
        state.bft.validate_votes(&block.header)?;

        // 1. validate the parent state root
        if block.header.state != self.storage.root()? {
            anyhow::bail!("Invalid parent state root");
        }

        // 2. update the accumulator with the signatures of the block
        let txs = block.extrinsic.txs();
        tracing::debug!("importing txs: {:?}", txs.len());
        let accumulator = self.accumulate(state.accumulator, &txs)?;
        if accumulator != block.header.accumulator {
            anyhow::bail!(
                "Invalid accumulator: parent={}, txs={}, accumulator={}",
                bs58::encode(state.accumulator).into_string(),
                txs.len(),
                bs58::encode(accumulator).into_string()
            );
        }

        // 3. stores the block to the storage
        let head = block.header.head();
        let mut commit = Commit::default();
        commit.insert(key::ACCUMULATOR_KEY, crypto::blake3(&accumulator).to_vec());
        commit.insert(key::BFT_KEY, postcard::to_allocvec(&state.bft)?);
        commit.insert(key::PRESENT_KEY, postcard::to_allocvec(&head)?);
        self.storage.commit(commit)?;
        self.storage.set_block(block)?;
        self.storage.set_txs(txs)?;
        Ok(())
    }
}
