//! Sync events

use zcore::ex::{Bridge, Receipt};

/// Sync events
pub enum Event {
    /// An incoming bridge transaction
    Bridge(Bridge),

    /// An incoming confirmation of the bridge transaction
    Receipt(Receipt),
}
