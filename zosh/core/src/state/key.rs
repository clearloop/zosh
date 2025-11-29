//! The state keys

macro_rules! to_key {
    ($key:expr) => {
        [
            $key, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0,
        ]
    };
}

/// The key for the BFT state
pub const BFT_KEY: [u8; 31] = to_key!(0);

/// The key for the history state
pub const HISTORY_KEY: [u8; 31] = to_key!(1);

/// The key for the Solana state
pub const SOL_KEY: [u8; 31] = to_key!(2);

/// The key for the Zcash state
pub const ZEC_KEY: [u8; 31] = to_key!(3);
