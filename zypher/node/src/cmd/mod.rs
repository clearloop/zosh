//! Command line interface for the ZypherBridge node

use clap::Parser;
use std::{path::PathBuf, sync::OnceLock};

mod dev;

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
}

impl App {
    /// Run the application
    pub fn run(&self) -> anyhow::Result<()> {
        match &self.command {
            Command::Dev(dev) => dev.run(&self.config),
        }?;

        Ok(())
    }
}

#[derive(Parser)]
pub enum Command {
    /// Development command
    #[clap(subcommand)]
    Dev(dev::Dev),
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
