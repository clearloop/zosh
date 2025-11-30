//! Utility traits for the core library

use anyhow::Result;

/// The message trait
pub trait Message {
    /// Get the message need to sign for the transaction
    fn message(&self) -> Vec<u8>;
}

/// convert the bytes to a fixed size array
pub trait FixedBytes {
    /// Convert the bytes to a 32 byte array
    fn bytes32(&self) -> Result<[u8; 32]>;

    /// Convert the bytes to a 64 byte array
    fn bytes64(&self) -> Result<[u8; 64]>;

    /// Convert the bytes to a fixed size array
    fn bytes<const N: usize>(&self) -> Result<[u8; N]>;
}

impl<T: AsRef<[u8]>> FixedBytes for T {
    fn bytes32(&self) -> Result<[u8; 32]> {
        self.bytes::<32>()
    }

    fn bytes64(&self) -> Result<[u8; 64]> {
        self.bytes::<64>()
    }

    fn bytes<const N: usize>(&self) -> Result<[u8; N]> {
        self.as_ref()
            .try_into()
            .map_err(|_| anyhow::anyhow!("expected {N} bytes, got {} bytes", self.as_ref().len()))
    }
}
