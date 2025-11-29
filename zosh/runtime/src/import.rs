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
    pub fn process(&self, extrinsic: &Extrinsic, mut state: State, head: Head) -> Result<Commit> {
        let mut commit = Commit::default();
        // 1. handle the tickets for zosh bft
        //
        // TODO: implement the tickets for zosh bft

        // 2. import the block to the history
        state.history.import(head)?;

        // 2. import the solana mint bundles
        if let Some(mint) = &extrinsic.mint {
            let bundle = mint.extrinsic.clone();
            let hash = crypto::blake3(&postcard::to_allocvec(&bundle)?);
            state.sol.insert(hash, bundle);
        }

        // 3. import the unlock bundles from zcash
        for unlock in &extrinsic.unlock {
            let bundle = unlock.extrinsic.clone();
            let hash = crypto::blake3(&postcard::to_allocvec(&bundle)?);
            state.zec.insert(hash, bundle);
        }

        // 4. process the receipts of mint bundles
        for receipt in &extrinsic.mint_receipts {
            state.sol.remove(&receipt.hash);
        }

        // 5. process the receipts of unlock bundles
        for receipt in &extrinsic.unlock_receipts {
            state.zec.remove(&receipt.hash);
        }

        // 6. commit the state
        commit.insert(key::BFT_KEY, postcard::to_allocvec(&state.bft)?);
        commit.insert(key::HISTORY_KEY, postcard::to_allocvec(&state.history)?);
        commit.insert(key::SOL_KEY, postcard::to_allocvec(&state.sol)?);
        commit.insert(key::ZEC_KEY, postcard::to_allocvec(&state.zec)?);
        Ok(commit)
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
