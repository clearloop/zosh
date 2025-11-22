//! Cache DB implementation

use std::sync::{Arc, Mutex};
use zcash_client_backend::{
    data_api::{
        chain::{error::Error, BlockCache, BlockSource},
        scanning::ScanRange,
    },
    proto::compact_formats::CompactBlock,
};
use zcash_client_sqlite::{error::SqliteClientError, BlockDb};
use zcash_protocol::consensus::BlockHeight;

/// Cache DB implementation
pub struct CacheDb {
    /// Block database connection
    block: Arc<Mutex<BlockDb>>,
}

impl BlockSource for CacheDb {
    type Error = SqliteClientError;

    fn with_blocks<F, WalletErrT>(
        &self,
        from_height: Option<BlockHeight>,
        limit: Option<usize>,
        with_block: F,
    ) -> Result<(), Error<WalletErrT, Self::Error>>
    where
        F: FnMut(CompactBlock) -> Result<(), Error<WalletErrT, Self::Error>>,
    {
        self.block
            .lock()
            .expect("Failed to lock block database connection")
            .with_blocks(from_height, limit, with_block)
    }
}

#[async_trait::async_trait]
impl BlockCache for CacheDb {
    fn get_tip_height(
        &self,
        range: Option<&ScanRange>,
    ) -> Result<Option<BlockHeight>, Self::Error> {
        todo!()
    }

    async fn read(&self, range: &ScanRange) -> Result<Vec<CompactBlock>, Self::Error> {
        let mut compact_blocks = vec![];
        if let Err(e) = self.with_blocks::<_, SqliteClientError>(
            Some(range.block_range().start),
            Some(range.len()),
            |block| {
                compact_blocks.push(block);
                Ok(())
            },
        ) {
            match e {
                Error::BlockSource(e) => return Err(e),
                Error::Wallet(e) => return Err(e),
                Error::Scan(_) => {
                    return Err(SqliteClientError::CorruptedData(
                        "this would never happen".to_string(),
                    ))
                }
            }
        }

        Ok(compact_blocks)
    }

    async fn insert(&self, compact_blocks: Vec<CompactBlock>) -> Result<(), Self::Error> {
        todo!()
    }

    async fn truncate(&self, block_height: BlockHeight) -> Result<(), Self::Error> {
        todo!()
    }

    async fn delete(&self, range: ScanRange) -> Result<(), Self::Error> {
        todo!()
    }
}

impl From<BlockDb> for CacheDb {
    fn from(block: BlockDb) -> Self {
        Self {
            block: Arc::new(Mutex::new(block)),
        }
    }
}
