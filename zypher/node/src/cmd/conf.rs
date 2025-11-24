//! Configuration command for the zyper bridge

use crate::{
    config::{Key, Network, Rpc},
    Config,
};
use anyhow::Result;
use runtime::signer::Keypair;
use std::{fs, path::Path};
use zcash::signer::GroupSigners;

const NOTE: &str = r#"
# Zyphers Configurations
#
# If you are a participant of our MPC protocol, please copy your generated shares
# at the [key.zcash] section.
"#;

/// Generate configuration file
pub fn generate(config: &Path) -> Result<()> {
    let target = config.join("config.toml");
    if target.exists() {
        return Err(anyhow::anyhow!("Configuration file already exists"));
    }

    // generate a default configuration file
    let config = Config {
        rpc: Rpc {
            solana: "https://api.mainnet-beta.solana.com".parse()?,
            lightwalletd: "http://127.0.0.1:9067".parse()?,
        },
        key: Key {
            zcash: bs58::encode(postcard::to_allocvec(&GroupSigners::new(3, 2)?)?).into_string(),
            solana: Keypair::new().to_base58_string(),
        },
        network: Network::Testnet,
    };
    fs::write(
        &target,
        format!("{}\n{}", NOTE, toml::to_string_pretty(&config)?),
    )?;
    println!(
        "Configuration file generated successfully in {}",
        target.display()
    );
    Ok(())
}
