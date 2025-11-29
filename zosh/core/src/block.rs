//! The block structure of zorch

use crate::Extrinsic;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// The block structure of zorch
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Block {
    /// The header of the block
    pub header: Header,

    /// The extrinsic of the block
    pub extrinsic: Extrinsic,
}

impl Block {
    /// Get the head of the block
    pub fn head(&self) -> Head {
        Head {
            height: self.header.height,
            hash: self.header.hash(),
        }
    }
}

/// The header structure of zorch
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Header {
    /// The height of the block
    pub height: u32,

    /// The parent block hash
    pub parent: [u8; 32],

    /// The merkle root of the state
    pub state: [u8; 32],

    /// The hash of the extrinsic
    pub extrinsic: [u8; 32],

    /// Signatures of the block (except the current field)
    pub votes: BTreeMap<[u8; 32], Vec<u8>>,
}

impl Header {
    /// Compute the hash of the header
    pub fn hash(&self) -> [u8; 32] {
        let mut data = self.height.to_le_bytes().to_vec();
        data.extend_from_slice(&self.parent);
        data.extend_from_slice(&self.state);
        data.extend_from_slice(&self.extrinsic);
        crypto::blake3(&data)
    }
}

/// The head of the block
pub struct Head {
    /// The height of the block
    pub height: u32,

    /// The parent block hash
    pub hash: [u8; 32],
}
