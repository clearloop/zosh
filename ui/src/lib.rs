// Library exports for the UI service

use std::net::SocketAddr;
pub use {config::Config, db::Database, error::AppError, hook::UIHook, sub::Subscriber};

mod config;
pub mod db;
mod error;
mod hook;
pub mod sub;
pub mod ui;
pub mod util;
pub mod web;

/// Spawn the UI service
pub fn spawn(db: Database, address: SocketAddr) {
    let querydb = db.clone();
    tokio::spawn(async move {
        if let Err(e) = sub::ui::subscribe(querydb).await {
            tracing::error!("Query ID subscriber error: {:?}", e);
        }
    });

    // Start web server
    tokio::spawn(async move {
        if let Err(e) = web::serve(address, db).await {
            tracing::error!("Web server error: {:?}", e);
        }
    });
}
