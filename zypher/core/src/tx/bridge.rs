//! The bridge transaction structure

/// The transaction structure of zyphers
pub struct Bridge {
    /// The recipient address
    pub recipient: Address,

    /// The amount of the transaction
    pub amount: u64,

    /// The source of the transaction
    pub source: Source,
}

/// The target chain address
pub enum Address {
    /// Solana address
    Solana([u8; 32]),

    /// Zcash orchard address
    ZcashO([u8; 43]),

    /// Zcash transparent address
    ZcashT([u8; 20]),
}

/// The transaction source
pub enum Source {
    /// Solana transaction signature
    Solana([u8; 64]),

    /// Zcash transaction id
    Zcash([u8; 32]),
}
