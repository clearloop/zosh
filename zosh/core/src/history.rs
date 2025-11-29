//! Block history related primitives

use crate::{block::Head, Block, EPOCH_LENGTH};
use anyhow::Result;
use std::collections::VecDeque;

/// The block history of the zorch network
pub struct History {
    /// The blocks of the network
    pub blocks: VecDeque<Head>,
    // TODO: add mmr here
}

impl History {
    /// Import a new block into the history
    pub fn import(&mut self, block: Block) -> Result<()> {
        self.blocks.push_back(block.head());
        while self.blocks.len() > EPOCH_LENGTH {
            self.blocks.pop_front();
        }

        Ok(())
    }
}
