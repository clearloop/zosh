//! Zcash related transactions

/// Spend bundle for unlocking zec
pub struct SpendBundle {
    /// The outputs of the frost wallet
    pub spend: Vec<(Vec<u8>, u64)>,

    /// Source signatures from solana
    pub source: Vec<Vec<u8>>,
}
