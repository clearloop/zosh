//! Development command for the zyper bridge

use anyhow::Result;
use clap::Parser;
use reddsa::frost::redjubjub::keys;
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
    ViewingKey {
        /// Group name of the signers
        #[clap(short, long, default_value = "default")]
        group: String,
        /// Address of the share
        #[clap(short, long, default_value = "default")]
        address: String,
    },
}

impl Dev {
    /// Run the development command
    pub fn run(&self, config: &Path) -> Result<()> {
        match self {
            Self::Dealer { name, min, max } => Self::dealers(config, name, *max, *min),
            Self::ViewingKey { group, address } => Self::viewing_key(config, group, address),
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

    /// Generate viewing key for a share
    pub fn viewing_key(config: &Path, group: &str, address: &str) -> Result<()> {
        let config = config.join(group).join(address).with_extension("toml");
        let share = fs::read_to_string(config)?;
        let key = toml::from_str::<Key>(&share)?;
        let signer = Signer::try_from(&key)?;
        let Some(zcash) = signer.zcash else {
            return Err(anyhow::anyhow!("No zcash key found"));
        };
        let ufvk = zcash.ufvk()?;
        println!(
            "Unified full viewing key: {}",
            ufvk.encode(&zcash::TestNetwork)
        );
        Ok(())
    }
}
