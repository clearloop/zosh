//! The state of the zosh network

use crate::{bft, Hash, Head};
use serde::{Deserialize, Serialize};

pub mod key;

/// The state of the zosh network
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct State {
    /// The BFT consensus state
    pub bft: bft::Bft,

    /// The present block head
    pub present: Head,

    /// The accumulator of all processed transactions
    pub accumulator: Hash,
}
