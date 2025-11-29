//! The block structure of zorch

use crate::Extrinsic;
use serde::{Deserialize, Serialize};

/// The block structure of zorch
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Block {
    /// The header of the block
    pub header: Header,

    /// The extrinsic of the block
    pub extrinsic: Extrinsic,
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
    pub votes: Vec<Vec<u8>>,
}
