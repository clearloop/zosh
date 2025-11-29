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
pub const PRESENT_KEY: [u8; 31] = to_key!(1);
