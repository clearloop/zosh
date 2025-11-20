//! Command line interface for the ZypherBridge node

use clap::Parser;
use std::{path::PathBuf, sync::OnceLock};

#[derive(Parser)]
pub struct App {
    /// Configuration file
    #[clap(short, long, default_value = default_config_dir())]
    pub config: PathBuf,

    /// Data directory
    #[clap(short, long, default_value = default_cache_dir())]
    pub cache: PathBuf,
}

fn default_config_dir() -> &'static str {
    static CONFIG_DIR: OnceLock<String> = OnceLock::new();
    CONFIG_DIR.get_or_init(|| {
        dirs::config_dir()
            .unwrap()
            .join(".zypher")
            .to_string_lossy()
            .into_owned()
    })
}

fn default_cache_dir() -> &'static str {
    static CACHE_DIR: OnceLock<String> = OnceLock::new();
    CACHE_DIR.get_or_init(|| {
        dirs::cache_dir()
            .unwrap()
            .join(".zypher")
            .to_string_lossy()
            .into_owned()
    })
}
