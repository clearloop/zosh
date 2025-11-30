//! Command line interface for the zorch node

use std::net::SocketAddr;

use crate::dev::Dev;
use anyhow::Result;
use clap::Parser;
use sync::{
    config::{Config, CACHE_DIR, CONFIG_DIR},
    solana, zcash,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// Command line interface for the ZorchBridge node
#[derive(Parser)]
pub struct App {
    #[clap(subcommand)]
    pub command: Command,

    /// Verbosity level
    #[clap(short, long, default_value = "0")]
    pub verbose: u8,
}

impl App {
    /// Run the application
    pub async fn run(&self) -> anyhow::Result<()> {
        self.init_tracing()?;
        self.create_dirs()?;
        match &self.command {
            Command::Dev { address } => Dev::new().await?.start(*address).await,
            Command::Solana(solana) => {
                let config = Config::load()?;
                solana.run(&config).await
            }
            Command::Zcash(zcash) => {
                let config = Config::load()?;
                zcash.run(&config).await
            }
        }?;

        Ok(())
    }

    fn create_dirs(&self) -> Result<()> {
        std::fs::create_dir_all(CONFIG_DIR.as_path())?;
        std::fs::create_dir_all(CACHE_DIR.as_path())?;
        Ok(())
    }

    fn init_tracing(&self) -> Result<()> {
        let verbosity = self.verbose;
        let level = match verbosity {
            0 => "info",
            1 => "debug",
            2 => "trace",
            _ => "trace",
        };

        // If verbose flag is set above 0, use it; otherwise use RUST_LOG or default
        let filter = if verbosity > 0 {
            EnvFilter::new(level)
        } else if let Ok(env) = std::env::var("RUST_LOG") {
            EnvFilter::new(env)
        } else {
            EnvFilter::new("info")
        };

        tracing_subscriber::registry()
            .with(filter)
            .with(tracing_subscriber::fmt::layer())
            .try_init()?;
        Ok(())
    }
}

/// Command line interface for the zorch node
#[derive(Parser)]
pub enum Command {
    /// Development commanm
    Dev {
        /// The address to bind the development node to
        #[clap(short, long, default_value = "127.0.0.1:1439")]
        address: SocketAddr,
    },

    /// Solana command
    #[clap(subcommand)]
    Solana(solana::Solana),

    /// Zcash command
    #[clap(subcommand)]
    Zcash(zcash::Zcash),
}
