//! Command line interface for the zyphers node

use clap::Parser;
use std::{path::PathBuf, sync::OnceLock};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

mod conf;
mod dev;
mod zcash;

/// Command line interface for the ZypherBridge node
#[derive(Parser)]
pub struct App {
    /// Configuration directory
    #[clap(short, long, default_value = default_config_dir())]
    pub config: PathBuf,

    /// Data directory
    #[clap(long, default_value = default_cache_dir())]
    pub cache: PathBuf,

    #[clap(subcommand)]
    pub command: Command,

    /// Verbosity level
    #[clap(short, long, default_value = "0")]
    pub verbose: u8,
}

impl App {
    /// Run the application
    pub async fn run(&self) -> anyhow::Result<()> {
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

        match &self.command {
            Command::Dev(dev) => dev.run(&self.config),
            Command::GenConf => conf::generate(&self.config),
            Command::Zcash(zcash) => zcash.run(&self.cache).await,
        }?;

        Ok(())
    }
}

/// Command line interface for the zyphers node
#[derive(Parser)]
pub enum Command {
    /// Development command
    #[clap(subcommand)]
    Dev(dev::Dev),

    /// Zcash command
    #[clap(subcommand)]
    Zcash(zcash::Zcash),

    /// Generate configuration file
    GenConf,
}

fn default_config_dir() -> &'static str {
    static CONFIG_DIR: OnceLock<String> = OnceLock::new();
    CONFIG_DIR.get_or_init(|| {
        dirs::config_dir()
            .unwrap()
            .join("zyphers")
            .to_string_lossy()
            .into_owned()
    })
}

fn default_cache_dir() -> &'static str {
    static CACHE_DIR: OnceLock<String> = OnceLock::new();
    CACHE_DIR.get_or_init(|| {
        dirs::cache_dir()
            .unwrap()
            .join("zyphers")
            .to_string_lossy()
            .into_owned()
    })
}
