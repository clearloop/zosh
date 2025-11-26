//! intergration tests

use anchor_client::Program;
use anyhow::Result;
use solana_sdk::{
    pubkey::Pubkey,
    signature::Keypair,
    signer::{EncodableKey, Signer},
};
use std::rc::Rc;
use zorch::{
    client::{util, ZorchClient},
    BridgeState,
};

#[tokio::test]
async fn test_connection() -> Result<()> {
    let test = Test::new().await?;
    let _ = test.client.program().rpc().get_latest_blockhash().await?;
    Ok(())
}

#[tokio::test]
async fn test_update_validators() -> Result<()> {
    let test = Test::new().await?;
    let state = test.bridge_state().await?;
    let validators = vec![test.payer()];
    let message = util::create_validators_message(state.nonce, &validators, 1);
    let signature = test.client.keypair.sign_message(&message);
    let signature = *signature.as_array();
    let _ = test
        .client
        .update_validators(vec![test.payer()], 1, vec![signature])
        .await?;
    Ok(())
}

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

    /// Get the bridge state
    pub async fn bridge_state(&self) -> Result<BridgeState> {
        if let Ok(state) = self.client.bridge_state().await {
            Ok(state)
        } else {
            self.client.initialize(vec![self.client.payer()], 1).await?;
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
