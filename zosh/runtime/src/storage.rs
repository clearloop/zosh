//! The storage of zosh

use anyhow::Result;
use std::sync::Arc;
use zcore::{state::key, Block, State, TrieKey};

/// The storage for the zosh bridge
pub trait Storage: Send + Sync + 'static {
    /// Batch the zosh state
    fn state(&self) -> Result<State> {
        let mut state = State::default();
        if let Some(value) = self.get(&key::ACCUMULATOR_KEY)? {
            state.accumulator = value
                .try_into()
                .map_err(|e| anyhow::anyhow!("Invalid accumulator: {e:?}"))?;
        }

        if let Some(value) = self.get(&key::BFT_KEY)? {
            state.bft = postcard::from_bytes(&value)?;
        }

        if let Some(value) = self.get(&key::PRESENT_KEY)? {
            state.present = postcard::from_bytes(&value)?;
        }

        Ok(state)
    }

    /// Get the value of the key
    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>>;

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

impl<S: Storage> Storage for Arc<S> {
    fn state(&self) -> Result<State> {
        self.as_ref().state()
    }

    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>> {
        self.as_ref().get(key)
    }

    fn commit(&self, commit: Commit) -> Result<()> {
        self.as_ref().commit(commit)
    }

    fn set_block(&self, block: Block) -> Result<()> {
        self.as_ref().set_block(block)
    }

    fn set_txs(&self, txs: Vec<Vec<u8>>) -> Result<()> {
        self.as_ref().set_txs(txs)
    }

    fn exists(&self, key: &[u8]) -> Result<bool> {
        self.as_ref().exists(key)
    }

    fn root(&self) -> Result<[u8; 32]> {
        self.as_ref().root()
    }
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
