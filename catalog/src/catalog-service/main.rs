mod validation;

use log::info;
use rust_catalog::startup::{Application, Settings};
use rust_common::{load_environment, mask_sensitive_url, setup_signal_handlers, HealthMonitor};
use std::{
    env,
    error::Error,
    io::{self, Write},
};
use validation::validate_catalog_dependencies;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    // Early boot diagnostic
    println!("BOOT: entering main");
    let _ = io::stdout().flush();

    // Load environment configuration FIRST, before initializing logger
    load_environment();

    // Initialize logger after loading environment (so RUST_LOG from .env is used)
    pretty_env_logger::init();

    // Log startup information
    info!(
        "🚀 Starting Rust Commerce Catalog Service v{}",
        env!("CARGO_PKG_VERSION")
    );
    info!("📋 Environment configuration:");
    info!(
        "  RUST_ENV: {}",
        env::var("RUST_ENV").unwrap_or_else(|_| "local".to_string())
    );
    info!(
        "  RUST_LOG: {}",
        env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string())
    );

    // Load settings from environment
    let settings = Settings::from_env();
    info!(
        "  MONGODB_URL: {}",
        mask_sensitive_url(&settings.mongodb_url)
    );
    info!("  NATS_URL: {}", settings.nats_url);
    info!("  CATALOG_DB_NAME: {}", settings.database_name);

    // Build the application
    let app = Application::build(settings).await?;

    // Setup signal handlers for graceful shutdown
    setup_signal_handlers().await?;

    // Validate dependencies
    validate_catalog_dependencies(&app.mongodb_client, &app.nats_client).await?;

    // Start health monitoring
    let health_monitor = HealthMonitor::new(app.mongodb_client.clone(), app.nats_client.clone());
    health_monitor.start_health_checks();

    info!("📊 Service startup completed successfully");

    // Run the application
    app.run().await
}
