//! ZorchBridge node

use clap::Parser;
use zorch_node::cmd::App;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let app = App::try_parse()?;
    app.run().await
}
