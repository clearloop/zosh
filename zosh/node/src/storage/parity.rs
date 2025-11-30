//! The parity storage implementation

use anyhow::Result;
use parity_db::{BTreeIterator, ColumnOptions, Db, Operation as Op, Options};
use runtime::storage::{Commit, Operation, Storage};
use std::path::PathBuf;
use zcore::Block;

/// The state column
pub const STATE_COLUMN: u8 = 0;

/// The block column
pub const BLOCK_COLUMN: u8 = 1;

/// The transaction column
pub const TRANSACTION_COLUMN: u8 = 2;

/// The parity database storage
pub struct Parity(Db);

impl Parity {
    /// Commit the genesis state
    pub fn is_empty(&self) -> Result<bool> {
        Ok(self.0.iter(STATE_COLUMN)?.next()?.is_none())
    }
}

impl Storage for Parity {
    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>> {
        self.0.get(STATE_COLUMN, key).map_err(Into::into)
    }

    fn commit(&self, commit: Commit) -> Result<()> {
        self.0
            .commit_changes(commit.ops().into_iter().map(|op| match op {
                Operation::Set(k, v) => (STATE_COLUMN, Op::Set(k.to_vec(), v)),
                Operation::Remove(k) => (STATE_COLUMN, Op::Dereference(k.to_vec())),
            }))?;
        Ok(())
    }

    fn set_block(&self, block: Block) -> Result<()> {
        self.0.commit_changes(vec![(
            BLOCK_COLUMN,
            Op::Set(block.header.hash().to_vec(), postcard::to_allocvec(&block)?),
        )])?;
        Ok(())
    }

    fn set_txs(&self, txs: Vec<Vec<u8>>) -> Result<()> {
        self.0.commit_changes(
            txs.into_iter()
                .map(|tx| (TRANSACTION_COLUMN, Op::Set(tx.to_vec(), vec![]))),
        )?;
        Ok(())
    }

    fn exists(&self, key: &[u8]) -> Result<bool> {
        self.0
            .get(TRANSACTION_COLUMN, key)
            .map(|value| value.is_some())
            .map_err(Into::into)
    }

    fn root(&self) -> Result<[u8; 32]> {
        let mut leaves = Vec::new();
        let iter = ParityIter(self.0.iter(STATE_COLUMN)?);
        for item in iter {
            let (_, value) = item?;
            leaves.push(value);
        }
        Ok(crypto::merkle::root(leaves))
    }
}

/// The iterator wrapper
pub struct ParityIter<'a>(BTreeIterator<'a>);

impl Iterator for ParityIter<'_> {
    type Item = Result<(Vec<u8>, Vec<u8>)>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map_err(Into::into).transpose()
    }
}

impl TryFrom<PathBuf> for Parity {
    type Error = anyhow::Error;

    fn try_from(path: PathBuf) -> Result<Self> {
        let options = Options {
            path,
            columns: vec![
                ColumnOptions {
                    btree_index: true,
                    ..Default::default()
                },
                ColumnOptions {
                    btree_index: true,
                    ..Default::default()
                },
                ColumnOptions {
                    btree_index: true,
                    ..Default::default()
                },
            ],
            sync_wal: true,
            sync_data: true,
            stats: true,
            salt: None,
            compression_threshold: Default::default(),
        };
        Ok(Parity(Db::open_or_create(&options)?))
    }
}
