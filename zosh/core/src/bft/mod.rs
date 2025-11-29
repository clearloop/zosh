//! zoshBFT related primitives

/// The zoshBFT consensus state
pub struct Bft {
    /// The validators of the BFT
    ///
    /// A set of ed25519 public keys
    pub validators: Vec<[u8; 32]>,

    /// The threshold for the BFT
    ///
    /// The number of validators that need to sign the block
    pub threshold: u8,

    /// The authoring randomness series
    pub series: Vec<[u8; 32]>,
}
