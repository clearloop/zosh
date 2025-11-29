//! The import interfaces for the runtime

use crate::{storage::Commit, Config, Runtime, Storage};
use anyhow::Result;
use zcore::{state::key, Block, Ed25519Signature, Extrinsic, Head, State};

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

        // validate the parent state root
        if block.header.state != self.storage.root() {
            anyhow::bail!("Invalid parent state root");
        }

        // import the block to the chain and update the state machine
        let head = block.header.head();
        let commit = self.process(&block.extrinsic, state, head)?;
        self.storage.commit(commit.ops())?;
        self.storage.set_block(block)?;
        Ok(())
    }

    /// Process the extrinsic to mutate the state machine
    ///
    /// then get the merkle root of the new state
    pub fn process(&self, _extrinsic: &Extrinsic, state: State, head: Head) -> Result<Commit> {
        let mut commit = Commit::default();
        // 1. handle the tickets for zosh bft
        //
        // TODO: implement the tickets for zosh bft

        // 7. commit the state
        commit.insert(key::BFT_KEY, postcard::to_allocvec(&state.bft)?);
        commit.insert(key::PRESENT_KEY, postcard::to_allocvec(&head)?);
        Ok(commit)
    }

    /// Validate the block
    ///
    /// After the validation we return the signature from our side, this
    /// will be used for complete the QC of the block.
    pub async fn validate(&mut self, block: Block) -> Result<Ed25519Signature> {
        let hash = block.header.hash();

        // TODO: validate the transactions inside of the block
        //
        // 1. solana mint bundles should satisfy the QC
        // 2. zcash unlock bundles should satisfy the QC
        // 3. validate the receipts and deduplicate our mempool

        self.sync
            .solana
            .sign_message(&hash)
            .map_err(Into::into)
            .map(|sig| *sig.as_array())
    }
}
