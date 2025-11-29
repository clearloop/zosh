//! The configuration of the zosh runtime

use crate::{Hook, Storage};

/// An aggregate configuration for the zosh runtime
pub trait Config {
    /// The hook type
    type Hook: Hook;

    /// The storage type
    type Storage: Storage;
}
