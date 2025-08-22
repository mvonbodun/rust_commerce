mod handlers;
mod model;
mod persistence;
mod validation;

use handlers::{
    create_item, delete_item, get_all_locations_by_sku, get_item, update_stock, Router,
};
use persistence::inventory_dao::InventoryDaoImpl;
use std::{env, error::Error, sync::Arc};

use log::{debug, error, info};
use rust_common::{
    load_environment, mask_sensitive_url, setup_signal_handlers, HealthMonitor, OperationTimer,
};
use validation::validate_inventory_dependencies;

use bson::doc;
use futures::StreamExt;
use model::InventoryItem;
use mongodb::{Client, Collection, IndexModel};

// Import common module for generated proto code
mod common {
    pub use shared_proto::common::*;
}

pub mod inventory_messages {
    include!(concat!(env!("OUT_DIR"), "/inventory_messages.rs"));

    // Re-export common types for backward compatibility
    pub use super::common::{Code, Status};
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    // Load environment configuration FIRST, before initializing logger
    load_environment();

    // Initialize logger after loading environment (so RUST_LOG from .env is used)
    pretty_env_logger::init();

    // Phase 1.1: Environment & Configuration Logging
    info!(
        "🚀 Starting Rust Commerce Inventory Service v{}",
        env!("CARGO_PKG_VERSION")
    );
    info!("📋 Environment configuration:");
    info!(
        "  RUST_ENV: {}",
        env::var("RUST_ENV").unwrap_or_else(|_| "development".to_string())
    );
    info!(
        "  RUST_LOG: {}",
        env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string())
    );

    // Get MongoDB URL
    let uri = env::var("MONGODB_URL").expect("MONGODB_URL must be set");
    info!("  MONGODB_URL: {}", mask_sensitive_url(&uri));

    // Get NATS URL
    let nats_url = env::var("NATS_URL").unwrap_or_else(|_| "nats://localhost:4222".to_string());
    info!("  NATS_URL: {nats_url}");

    // Phase 1.2: MongoDB Connection Logging
    info!("🔗 Connecting to MongoDB...");
    let client = match Client::with_uri_str(&uri).await {
        Ok(client) => {
            info!("✅ Successfully connected to MongoDB");
            // Test the connection
            match client.list_database_names().await {
                Ok(databases) => {
                    debug!("📋 Available databases: {databases:?}");
                    client
                }
                Err(e) => {
                    error!("❌ Failed to list databases: {e}");
                    return Err(e.into());
                }
            }
        }
        Err(e) => {
            error!("❌ Failed to connect to MongoDB: {e}");
            return Err(e.into());
        }
    };
    let database = client.database("db_inventory");
    info!("📊 Using database: db_inventory");

    // Phase 1.3: Inventory Collection & Index Setup Logging
    info!("📦 Setting up inventory collection...");
    let inventory_coll: Collection<InventoryItem> = database.collection("inventory");
    let indexes = vec![
        IndexModel::builder()
            .keys(doc! { "sku": 1, "location": 1})
            .options(
                mongodb::options::IndexOptions::builder()
                    .unique(true)
                    .build(),
            )
            .build(),
        IndexModel::builder().keys(doc! { "sku": 1 }).build(),
    ];
    info!("🔍 Creating {} inventory indexes...", indexes.len());
    match inventory_coll.create_indexes(indexes).await {
        Ok(result) => {
            info!(
                "✅ Created {} inventory indexes successfully",
                result.index_names.len()
            );
            debug!("Inventory indexes: sku+location (unique), sku");
        }
        Err(e) => {
            error!("❌ Failed to create inventory indexes: {e}");
            return Err(e.into());
        }
    }

    // Phase 2.1: DAO Setup Logging
    info!("🏗️  Initializing data access objects...");
    let inventory_dao = Arc::new(InventoryDaoImpl::new(inventory_coll));
    debug!("✅ Inventory DAO initialized");

    // Phase 2.2: Router Setup Logging
    info!("🛣️  Setting up message router...");
    let mut router = Router::new();
    router
        .add_route(
            "create_item".to_owned(),
            Box::new(|d, m| Box::pin(create_item(d, m))),
        )
        .add_route(
            "get_item".to_owned(),
            Box::new(|d, m| Box::pin(get_item(d, m))),
        )
        .add_route(
            "get_all_locations_by_sku".to_owned(),
            Box::new(|d, m| Box::pin(get_all_locations_by_sku(d, m))),
        )
        .add_route(
            "delete_item".to_owned(),
            Box::new(|d, m| Box::pin(delete_item(d, m))),
        )
        .add_route(
            "update_stock".to_owned(),
            Box::new(|d, m| Box::pin(update_stock(d, m))),
        );

    let route_count = 5;
    info!("✅ Configured {route_count} inventory routes");
    debug!("Inventory routes: create_item, get_item, get_all_locations_by_sku, delete_item, update_stock");

    // Phase 1.4: NATS Connection Logging
    info!("🔗 Connecting to NATS server: {nats_url}");
    let nats_client = match async_nats::connect(&nats_url).await {
        Ok(client) => {
            info!("✅ Successfully connected to NATS");
            client
        }
        Err(e) => {
            error!("❌ Failed to connect to NATS: {e}");
            return Err(e.into());
        }
    };

    // Phase 4: Setup signal handlers for graceful shutdown
    setup_signal_handlers().await?;
    debug!("✅ Signal handlers configured");

    // Phase 4: Validate dependencies
    validate_inventory_dependencies(&client, &nats_client).await?;

    // Phase 3.1: Queue Subscription Logging
    info!("📡 Subscribing to NATS queue: inventory.*");
    let requests = match nats_client
        .queue_subscribe("inventory.*", "queue".to_owned())
        .await
    {
        Ok(subscription) => {
            info!("✅ Successfully subscribed to inventory.* queue");
            subscription
        }
        Err(e) => {
            error!("❌ Failed to subscribe to NATS queue: {e}");
            return Err(e.into());
        }
    };

    let routes = Arc::new(router.route_map);

    // Phase 4: Start health monitoring
    let health_monitor = HealthMonitor::new(client.clone(), nats_client.clone());
    health_monitor.start_health_checks();
    debug!("✅ Health monitoring started");

    info!("🚀 Inventory service is ready and listening for requests");
    info!("📊 Service startup completed successfully");

    // Phase 3.2: Request Processing Logging
    requests
        .for_each_concurrent(25, |request| {
            let od = inventory_dao.clone();
            let routes = routes.clone();
            let client_clone = nats_client.clone();

            async move {
                let subject_parts: Vec<&str> = request.subject.split('.').collect();
                if subject_parts.len() < 2 {
                    error!("Invalid subject format: {}", request.subject);
                    return;
                }

                let operation = subject_parts[1].to_string();
                debug!(
                    "📨 Processing inventory operation: {} from subject: {}",
                    operation, request.subject
                );

                // Phase 5: Use OperationTimer for performance monitoring
                let op_name = format!("inventory.{operation}");
                let _timer = OperationTimer::new(&op_name);

                let result = if let Some(handler) = routes.get(&operation) {
                    let response = handler.call(od, request).await;
                    // Publish response
                    if let Err(e) = client_clone
                        .publish(response.subject, response.payload)
                        .await
                    {
                        error!("❌ Failed to publish response: {e}");
                        Err(Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
                    } else {
                        Ok(())
                    }
                } else {
                    error!("No handler found for operation: {operation}");
                    Ok(())
                };

                match result {
                    Ok(_) => {
                        _timer.log_elapsed("debug");
                    }
                    Err(e) => {
                        _timer.log_elapsed("error");
                        error!("❌ Error details: {e:?}");
                    }
                }
            }
        })
        .await;

    Ok(())
}
