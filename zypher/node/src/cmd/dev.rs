//! Development command for the zyper bridge

use anyhow::Result;
use clap::Parser;
use reddsa::frost::redpallas::keys;
use runtime::{
    config::Key,
    signer::{Keypair, Signer},
};
use solana_signer::Signer as _;
use std::{fs, path::Path};
use zcash::signer::ShareSigner;

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
    Info {
        /// Group name of the signers
        #[clap(short, long, default_value = "default")]
        group: String,
    },
}

impl Dev {
    /// Run the development command
    pub fn run(&self, config: &Path) -> Result<()> {
        match self {
            Self::Dealer { name, min, max } => Self::dealers(config, name, *max, *min),
            Self::Info { group } => Self::info(config, group),
        }
    }

    /// Generate dealers for the MPC protocol
    pub fn dealers(config: &Path, name: &str, max: u16, min: u16) -> Result<()> {
        let rng = rand_core::OsRng;
        let (shares, package) =
            keys::generate_with_dealer(max, min, keys::IdentifierList::Default, rng)?;
        let signers = shares
            .iter()
            .map(|(ident, share)| Signer {
                zcash: Some(ShareSigner {
                    identifier: *ident,
                    rjpackage: package.clone(),
                    rjshare: share.clone(),
                }),
                solana: Keypair::new(),
            })
            .collect::<Vec<_>>();

        // write the signers to the config folder
        let group = config.join(name);
        fs::create_dir_all(&group)?;
        for signer in signers {
            let spub = signer.solana.pubkey().to_string() + ".toml";
            let key: Key = signer.try_into()?;
            fs::write(group.join(&spub), toml::to_string(&key)?)?;
        }

        println!("Signers generated successfully in {}", group.display());
        Ok(())
    }

    /// Get the info of a group
    pub fn info(config: &Path, group: &str) -> Result<()> {
        let config = config.join(group);
        for entry in fs::read_dir(config)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                continue;
            }

            let file = fs::read_to_string(path)?;
            let key = toml::from_str::<Key>(&file)?;
            let signer = Signer::try_from(&key)?;
            let Some(zcash) = signer.zcash else {
                continue;
            };

            let address = zcash.external_address()?;
            println!(
                "External address: {}",
                hex::encode(&address.to_raw_address_bytes())
            );

            let uaddr = zcash.unified_address()?;
            println!("Unified address: {}", uaddr.encode(&zcash::TestNetwork));

            let ufvk = zcash.ufvk()?;
            println!(
                "Unified full viewing key: {}",
                ufvk.encode(&zcash::TestNetwork)
            );

            break;
        }
        Ok(())
    }
}
