//! Configuration command for the zyper bridge

use crate::{
    config::{Key, Rpc},
    Config,
};
use anyhow::Result;
use runtime::signer::Keypair;
use std::{fs, path::PathBuf};

const NOTE: &str = r#"
# Zyphers Configurations
#
# If you are a participant of our MPC protocol, please copy your generated shares
# at the [key.zcash] section.
"#;

/// Generate configuration file
pub fn generate(config: &PathBuf) -> Result<()> {
    let target = config.join("config.toml");
    if target.exists() {
        return Err(anyhow::anyhow!("Configuration file already exists"));
    }

    // generate a default configuration file
    let config = Config {
        sync: Rpc {
            solana: "https://api.mainnet-beta.solana.com".parse()?,
            zcash: "https://api.zcashexplorer.app".parse()?,
        },
        key: Key {
            zcash: None,
            solana: Keypair::new().to_base58_string(),
        },
    };
    fs::write(
        &target,
        &format!("{}\n{}", NOTE, toml::to_string_pretty(&config)?),
    )?;
    println!(
        "Configuration file generated successfully in {}",
        target.display()
    );
    Ok(())
}
