//! Crypto primitives for the zosh bridge

pub mod ed25519;
pub mod merkle;

/// Compute the Blake3 hash of the data
pub fn blake3(data: &[u8]) -> [u8; 32] {
    let mut hasher = blake3::Hasher::new();
    hasher.update(data);
    hasher.finalize().into()
}
