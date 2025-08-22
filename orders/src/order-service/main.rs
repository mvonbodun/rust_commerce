pub mod model;

mod handlers;
mod persistence;
mod validation;

use handlers::{create_order, delete_order, get_order, Router};
use log::{debug, error, info};
use persistence::orders_dao::{OrdersDao, OrdersDaoImpl};
use std::{env, error::Error, sync::Arc};

use rust_common::{
    load_environment, mask_sensitive_url, setup_signal_handlers, HealthMonitor, OperationTimer,
};
use validation::validate_orders_dependencies;

use bson::doc;
use model::Order;
use mongodb::{Client, Collection, IndexModel};

use futures::StreamExt;

// Import common module for generated proto code
mod common {
    pub use shared_proto::common::*;
}

pub mod order_messages {
    include!(concat!(env!("OUT_DIR"), "/order_messages.rs"));

    // Re-export common types for backward compatibility
    pub use super::common::{Code, Status};
}

#[derive(Clone)]
pub struct AppState {
    pub orders_dao: Arc<dyn OrdersDao + Send + Sync>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    // Load environment configuration FIRST, before initializing logger
    load_environment();

    // Initialize logger after loading environment (so RUST_LOG from .env is used)
    pretty_env_logger::init();

    // Phase 1.1: Environment & Configuration Logging
    info!(
        "🚀 Starting Rust Commerce Orders Service v{}",
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
    let database = client.database("db_orders");
    info!("📊 Using database: db_orders");

    // Phase 1.3: Orders Collection & Index Setup Logging
    info!("📦 Setting up orders collection...");
    let orders_coll: Collection<Order> = database.collection("orders");
    let indexes = vec![
        IndexModel::builder()
            .keys(doc! { "order_id": 1})
            .options(
                mongodb::options::IndexOptions::builder()
                    .unique(true)
                    .build(),
            )
            .build(),
        IndexModel::builder()
            .keys(doc! { "customer_id": 1 })
            .build(),
        IndexModel::builder().keys(doc! { "status": 1 }).build(),
    ];
    info!("🔍 Creating {} order indexes...", indexes.len());
    match orders_coll.create_indexes(indexes).await {
        Ok(result) => {
            info!(
                "✅ Created {} order indexes successfully",
                result.index_names.len()
            );
            debug!("Order indexes: order_id (unique), customer_id, status");
        }
        Err(e) => {
            error!("❌ Failed to create order indexes: {e}");
            return Err(e.into());
        }
    }

    // Phase 2.1: DAO Setup Logging
    info!("🏗️  Initializing data access objects...");
    let orders_dao = Arc::new(OrdersDaoImpl::new(orders_coll));
    debug!("✅ Orders DAO initialized");

    // Phase 2.2: Router Setup Logging
    info!("🛣️  Setting up message router...");
    let mut router = Router::new();
    router
        .add_route(
            "create_order".to_owned(),
            Box::new(|a, b, c| Box::pin(create_order(a, b, c))),
        )
        .add_route(
            "get_order".to_owned(),
            Box::new(|a, b, c| Box::pin(get_order(a, b, c))),
        )
        .add_route(
            "delete_order".to_owned(),
            Box::new(|a, b, c| Box::pin(delete_order(a, b, c))),
        );

    let route_count = 3;
    info!("✅ Configured {route_count} order routes");
    debug!("Order routes: create_order, get_order, delete_order");

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
    validate_orders_dependencies(&client, &nats_client).await?;

    // Phase 3.1: Queue Subscription Logging
    info!("📡 Subscribing to NATS queue: orders.*");
    let requests = match nats_client
        .queue_subscribe("orders.*", "queue".to_owned())
        .await
    {
        Ok(subscription) => {
            info!("✅ Successfully subscribed to orders.* queue");
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

    info!("🚀 Orders service is ready and listening for requests");
    info!("📊 Service startup completed successfully");

    // Phase 3.2: Request Processing Logging
    requests
        .for_each_concurrent(25, |request| {
            let od = orders_dao.clone();
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
                    "📨 Processing orders operation: {} from subject: {}",
                    operation, request.subject
                );

                // Phase 5: Use OperationTimer for performance monitoring
                let _timer = OperationTimer::new(&format!("orders.{operation}"));

                let result: Result<(), Box<dyn std::error::Error + Send + Sync>> =
                    if let Some(handler) = routes.get(&operation) {
                        handler(client_clone, od, request).await;
                        Ok(())
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
