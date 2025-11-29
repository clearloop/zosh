//! The state of the zosh network

use crate::{bft, Hash, History};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub mod key;
pub mod sol;

/// The state of the zosh network
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct State {
    /// The BFT consensus state
    pub bft: bft::Bft,

    /// The history of the network
    pub history: History,

    /// The solana mint bundles
    pub sol: BTreeMap<Hash, sol::MintBundle>,

    /// The zcash unlock bundles
    pub zec: BTreeMap<Hash, Vec<u8>>,
}
