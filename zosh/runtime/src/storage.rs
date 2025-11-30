//! The storage of zosh

use anyhow::Result;
use zcore::{Block, State, TrieKey};

/// The storage for the zosh bridge
pub trait Storage: Send + Sync + 'static {
    /// Batch the zosh state
    fn state(&self) -> State;

    /// Batch the operations to the storage
    fn commit(&self, commit: Commit) -> Result<()>;

    /// Set the block to the storage
    fn set_block(&self, block: Block) -> Result<()>;

    /// Set the transactions to the storage
    ///
    /// TODO: use reference instead of cloning
    fn set_txs(&self, txs: Vec<Vec<u8>>) -> Result<()>;

    /// Check if transaction id exists in the storage
    fn exists(&self, key: &[u8]) -> Result<bool>;

    /// Get the root of the state
    fn root(&self) -> Result<[u8; 32]>;
}

/// Commit builder
#[derive(Default)]
pub struct Commit {
    /// The insert operations
    insert: Vec<(TrieKey, Vec<u8>)>,

    /// The remove operations
    remove: Vec<TrieKey>,
}

impl Commit {
    /// Insert the value of the key
    pub fn insert(&mut self, key: TrieKey, value: Vec<u8>) -> &mut Self {
        self.insert.push((key, value));
        self
    }

    /// Remove the value of the key
    #[allow(unused)]
    pub fn remove(&mut self, key: TrieKey) -> &mut Self {
        self.remove.push(key);
        self
    }

    /// Build the commit
    pub fn ops(&self) -> Vec<Operation> {
        let mut ops = Vec::new();
        for (key, value) in &self.insert {
            ops.push(Operation::Set(*key, value.clone()));
        }
        for key in &self.remove {
            ops.push(Operation::Remove(*key));
        }
        ops
    }
}

/// The operation of the storage
pub enum Operation {
    /// Set the value of the key
    Set([u8; 31], Vec<u8>),

    /// Remove the value of the key
    Remove([u8; 31]),
}
