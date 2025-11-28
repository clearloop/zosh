//! The intent for non-VM chains

use serde::{Deserialize, Serialize};

/// The intent for non-VM chains
///
/// TODO: this is not used at the moment, waiting for zashi
/// to support the QRcode solution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Intent {
    /// The intent to bridge to the other chain
    Bridge {
        /// The recipient address on the other chain
        recipient: String,
    },
}
