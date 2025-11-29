//! Block history related primitives

use crate::{block::Head, EPOCH_LENGTH};
use anyhow::Result;
use serde::{Deserialize, Serialize};

/// The block history of the zorch network
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct History {
    /// The blocks of the network
    pub blocks: Vec<Head>,
    // TODO: add mmr here
}

impl History {
    /// Import a new block into the history
    pub fn import(&mut self, head: Head) -> Result<()> {
        self.blocks.push(head);
        while self.blocks.len() > EPOCH_LENGTH {
            self.blocks = self.blocks.split_off(1);
        }

        Ok(())
    }
}
