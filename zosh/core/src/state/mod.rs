//! The state of the zosh network

use crate::bft;

pub mod key;
pub mod sol;

/// The state of the zosh network
pub struct State {
    /// The solana mint bundles
    pub sol: Vec<sol::MintBundle>,

    /// The zcash unlock bundles
    pub zec: Vec<Vec<u8>>,

    /// The BFT consensus state
    pub bft: bft::Bft,
}
