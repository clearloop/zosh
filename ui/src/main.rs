//! UI Web Service for Zosh Bridge
//!
//! This service subscribes to Zosh blocks via RPC, stores them in SQLite,
//! and provides a web API to query bridge transactions.

mod config;
mod db;
mod subscriber;
mod web;

use anyhow::Result;
use config::Config;
use db::Database;
use subscriber::Subscriber;
use tokio::signal;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "ui=info,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting Zosh UI Web Service");

    // Load configuration
    let config = Config::load()?;
    tracing::info!("Configuration loaded:");
    tracing::info!("  RPC URL: {}", config.rpc_url);
    tracing::info!("  Database: {}", config.db_path);
    tracing::info!("  Listen address: {}", config.listen_addr);

    // Initialize database
    let db = Database::new(&config.db_path)?;
    db.init()?;
    tracing::info!("Database initialized");

    // Clone database for web server
    let web_db = db.clone();
    let subscriber = Subscriber::new(config.rpc_url.clone(), db);
    let subscriber_handle = tokio::spawn(async move {
        loop {
            tracing::info!("Starting block subscriber...");
            if let Err(e) = subscriber.run().await {
                tracing::error!("Subscriber error: {:?}", e);
                tracing::info!("Reconnecting in 5 seconds...");
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            }
        }
    });

    // Start web server
    let web_handle = tokio::spawn(async move {
        if let Err(e) = web::serve(config.listen_addr, web_db).await {
            tracing::error!("Web server error: {:?}", e);
        }
    });

    // Wait for shutdown signal
    match signal::ctrl_c().await {
        Ok(()) => {
            tracing::info!("Received shutdown signal, stopping...");
        }
        Err(err) => {
            tracing::error!("Failed to listen for shutdown signal: {:?}", err);
        }
    }

    // Abort tasks
    subscriber_handle.abort();
    web_handle.abort();
    tracing::info!("Shutdown complete");
    Ok(())
}
