//! The author interfaces for the runtime

use crate::{Config, Runtime};
use anyhow::Result;
use zcore::Block;

impl<C: Config> Runtime<C> {
    /// Author a new block
    pub async fn author(&mut self) -> Result<Block> {
        Ok(Block::default())
    }
}
