//! ZorchBridge node

use clap::Parser;
use zosh_node::cmd::App;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let app = App::parse();
    app.run().await
}
