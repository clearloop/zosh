//! Development command for the zyper bridge

use anyhow::Result;
use clap::Parser;
use reddsa::frost::redjubjub::keys;
use runtime::{
    config::Key,
    signer::{Keypair, Signer, ZcashSharedSigner},
};
use solana_signer::Signer as _;
use std::{fs, path::Path};

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
    pub fn run(&self, config: &Path) -> Result<()> {
        match self {
            Self::Dealer { name, min, max } => Self::dealers(config, name, *max, *min),
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
                zcash: Some(ZcashSharedSigner {
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
}
