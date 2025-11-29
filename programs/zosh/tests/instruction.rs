//! integration tests

use anchor_client::Program;
use anyhow::Result;
use solana_sdk::{pubkey::Pubkey, signature::Keypair, signer::EncodableKey};
use std::rc::Rc;
use zosh::{client::ZoshClient, types::MintEntry, BridgeState};

#[tokio::test]
async fn test_connection() -> Result<()> {
    let test = Test::new().await?;
    let _ = test.client.program().rpc().get_latest_blockhash().await?;
    Ok(())
}

#[tokio::test]
async fn test_mint() -> Result<()> {
    let test = Test::new().await?;
    let mints = vec![MintEntry {
        recipient: test.payer(),
        amount: 42,
    }];
    let _ = test.client.send_mint(mints).await?;
    Ok(())
}

/// Test environment
pub struct Test {
    /// Anchor client
    pub client: ZoshClient,
}

impl Test {
    /// Create a new test environment
    pub async fn new() -> Result<Self> {
        let home = dirs::home_dir().ok_or(anyhow::anyhow!("Home directory not found"))?;
        let payer = Keypair::read_from_file(home.join(".config/solana/id.json"))
            .map_err(|e| anyhow::anyhow!("Error reading `~/.config/solana/id.json`: {}", e))?;
        let client = ZoshClient::new(
            "http://localhost:8899".into(),
            "ws://localhost:8900".into(),
            payer,
        )?;

        Ok(Self { client })
    }

    /// Get the bridge state
    pub async fn bridge_state(&self) -> Result<BridgeState> {
        if let Ok(state) = self.client.bridge_state().await {
            Ok(state)
        } else {
            self.client.initialize().await?;
            self.client.bridge_state().await
        }
    }

    /// Get the payer's public key
    pub fn payer(&self) -> Pubkey {
        self.client.program().payer()
    }

    /// Get the program client
    pub fn program(&self) -> &Program<Rc<Keypair>> {
        self.client.program()
    }
}
