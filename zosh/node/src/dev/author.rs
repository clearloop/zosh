//! The development node authoring service

use crate::dev::Development;
use anyhow::Result;
use runtime::Runtime;

/// Start the authoring service
pub async fn start(_runtime: Runtime<Development>) -> Result<()> {
    Ok(())
}
