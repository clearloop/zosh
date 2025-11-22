//! Cache DB implementation

use prost::Message;
use rusqlite::{params, Connection};
use std::{
    path::Path,
    sync::{Arc, Mutex},
};
use zcash_client_backend::{
    data_api::{
        chain::{error::Error, BlockCache, BlockSource},
        scanning::ScanRange,
    },
    proto::compact_formats::CompactBlock,
};
use zcash_client_sqlite::error::SqliteClientError;
use zcash_protocol::consensus::BlockHeight;

/// Cache DB implementation
pub struct BlockDb {
    /// Block database connection
    block: Arc<Mutex<Connection>>,
}

impl BlockDb {
    /// Create a new block database from a path
    pub fn for_path(path: impl AsRef<Path>) -> Result<Self, SqliteClientError> {
        let connection = Connection::open(path.as_ref())?;
        connection.execute(
            "CREATE TABLE IF NOT EXISTS compactblocks (
                height INTEGER PRIMARY KEY,
                data BLOB NOT NULL
            )",
            [],
        )?;

        Ok(Self {
            block: Arc::new(Mutex::new(connection)),
        })
    }
}

impl BlockSource for BlockDb {
    type Error = SqliteClientError;

    fn with_blocks<F, WalletErrT>(
        &self,
        from_height: Option<BlockHeight>,
        limit: Option<usize>,
        mut with_block: F,
    ) -> Result<(), Error<WalletErrT, Self::Error>>
    where
        F: FnMut(CompactBlock) -> Result<(), Error<WalletErrT, Self::Error>>,
    {
        let source = self.block.lock().unwrap();
        // Fetch the CompactBlocks we need to scan
        let mut stmt_blocks = source
            .prepare(
                "SELECT height, data FROM compactblocks
        WHERE height >= ?
        ORDER BY height ASC LIMIT ?",
            )
            .map_err(|e| Error::BlockSource(e.into()))?;

        let mut rows = stmt_blocks
            .query(params![
                from_height.map_or(0u32, u32::from),
                limit
                    .and_then(|l| u32::try_from(l).ok())
                    .unwrap_or(u32::MAX)
            ])
            .map_err(|e| Error::BlockSource(e.into()))?;

        // Only look for the `from_height` in the scanned blocks if it is set.
        let mut from_height_found = from_height.is_none();
        while let Some(row) = rows.next().map_err(|e| Error::BlockSource(e.into()))? {
            let height =
                BlockHeight::from_u32(row.get(0).map_err(|e| Error::BlockSource(e.into()))?);
            if !from_height_found {
                // We will only perform this check on the first row.
                let from_height = from_height.expect("can only reach here if set");
                if from_height != height {
                    return Err(Error::BlockSource(
                        SqliteClientError::CacheMiss(from_height).into(),
                    ));
                } else {
                    from_height_found = true;
                }
            }

            let data: Vec<u8> = row.get(1).map_err(|e| Error::BlockSource(e.into()))?;
            let block =
                CompactBlock::decode(&data[..]).map_err(|e| Error::BlockSource(e.into()))?;
            if block.height() != height {
                return Err(Error::BlockSource(
                    SqliteClientError::CorruptedData(format!(
                        "Block height {} did not match row's height field value {}",
                        block.height(),
                        height
                    ))
                    .into(),
                ));
            }

            with_block(block)?;
        }

        if !from_height_found {
            let from_height = from_height.expect("can only reach here if set");
            return Err(Error::BlockSource(
                SqliteClientError::CacheMiss(from_height).into(),
            ));
        }

        Ok(())
    }
}

#[async_trait::async_trait]
impl BlockCache for BlockDb {
    fn get_tip_height(
        &self,
        range: Option<&ScanRange>,
    ) -> Result<Option<BlockHeight>, Self::Error> {
        let source = self.block.lock().unwrap();
        let mut stmt = source.prepare("SELECT MAX(height) FROM compactblocks")?;
        let mut rows = stmt.query([])?;
        let Some(row) = rows.next()? else {
            return Ok(None);
        };
        let height = row
            .get(0)
            .unwrap_or(0)
            .max(range.map_or(0, |r| r.block_range().end.into()));
        Ok(Some(BlockHeight::from_u32(height)))
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
        let mut source = self.block.lock().unwrap();
        let tx = source.transaction()?;
        for block in compact_blocks {
            let height: u32 = block.height().into();
            tx.execute(
                "INSERT INTO compactblocks (height, data) VALUES (?, ?)",
                params![height, block.encode_to_vec()],
            )?;
        }
        tx.commit()?;
        Ok(())
    }

    async fn truncate(&self, block_height: BlockHeight) -> Result<(), Self::Error> {
        let source = self.block.lock().unwrap();
        let height: u32 = block_height.into();
        source.execute(
            "DELETE FROM compactblocks WHERE height > ?",
            params![height],
        )?;
        Ok(())
    }

    async fn delete(&self, range: ScanRange) -> Result<(), Self::Error> {
        let mut source = self.block.lock().unwrap();
        let tx = source.transaction()?;
        let start: u32 = range.block_range().start.into();
        let end: u32 = range.block_range().end.into();
        tx.execute(
            "DELETE FROM compactblocks WHERE height >= ? AND height <= ?",
            params![start, end],
        )?;
        tx.commit()?;
        Ok(())
    }
}
