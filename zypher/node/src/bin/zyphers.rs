//! ZypherBridge node

use clap::Parser;
use zypher_node::cmd::App;

fn main() -> anyhow::Result<()> {
    let app = App::try_parse()?;
    app.run()
}
