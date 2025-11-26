//! intergration tests

use anyhow::Result;
use solana_sdk::{signature::Keypair, signer::EncodableKey};
use zorch::client::ZorchClient;

mod internal;

/// Test environment
pub struct Test {
    /// Anchor client
    pub client: ZorchClient,
}

impl Test {
    /// Create a new test environment
    pub async fn new() -> Result<Self> {
        let home = dirs::home_dir().ok_or(anyhow::anyhow!("Home directory not found"))?;
        let payer = Keypair::read_from_file(home.join(".config/solana/id.json"))
            .map_err(|e| anyhow::anyhow!("Error reading `~/.config/solana/id.json`: {}", e))?;
        let client = ZorchClient::new(
            "http://localhost:8899".into(),
            "ws://localhost:8900".into(),
            payer,
        )?;
        Ok(Self { client })
    }
}

#[tokio::test]
async fn test_connection() -> Result<()> {
    let test = Test::new().await?;
    let _ = test.client.program().rpc().get_latest_blockhash().await?;
    Ok(())
}
