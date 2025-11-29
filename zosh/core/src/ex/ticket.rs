//! Tickets for rotating the validators

use serde::{Deserialize, Serialize};

/// The ticket for rotating the validators
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Ticket {
    /// The id of the ticket
    pub id: u8,

    /// The validator that is being rotated
    pub validator: [u8; 32],
}
