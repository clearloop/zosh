//! ZorchBridge node

use clap::Parser;
use zosh_node::cmd::App;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize rustls crypto provider for TLS connections
    let _ = rustls::crypto::ring::default_provider().install_default();
    
    let app = App::parse();
    app.run().await
}
