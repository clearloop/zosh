//! The node implementations

use crate::storage::Parity;
pub use dev::Dev;
use runtime::Config;

mod dev;

/// The development node configuration
pub struct Development;

impl Config for Development {
    type Hook = dev::DevHook;
    type Storage = Parity;
}
