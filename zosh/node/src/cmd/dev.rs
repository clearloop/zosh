//! Development command for the zyper bridge

use anyhow::Result;
use clap::Parser;

/// Development command for the zyper bridge
#[derive(Parser)]
pub enum Dev {
    /// Generate signers for the MPC protocol
    Dealer {
        /// Group name of the signers
        #[clap(short, long, default_value = "default")]
        name: String,
        /// Maximum number of signers
        #[clap(long, default_value = "3")]
        max: u16,
        /// Minimum number of signers
        #[clap(long, default_value = "2")]
        min: u16,
    },
}

impl Dev {
    /// Run the development command
    pub async fn run(&self) -> Result<()> {
        Ok(())
    }
}
