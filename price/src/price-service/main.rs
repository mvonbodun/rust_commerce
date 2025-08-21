mod handlers;
mod model;
mod persistence;

use handlers::{
    create_offer, delete_offer, get_best_offer_price, get_best_offer_prices, get_offer, Router,
};
use persistence::offer_dao::OfferDaoImpl;
use std::{env, error::Error, sync::Arc};

use log::{debug, error, info};
use rust_common::{
    load_environment, mask_sensitive_url, setup_signal_handlers, validate_price_dependencies,
    HealthMonitor, OperationTimer,
};

use bson::doc;
use futures::StreamExt;
use model::Offer;
use mongodb::{Client, Collection, IndexModel};

// Import common module for generated proto code
mod common {
    pub use shared_proto::common::*;
}

pub mod offer_messages {
    include!(concat!(env!("OUT_DIR"), "/offer_messages.rs"));

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
        "🚀 Starting Rust Commerce Price Service v{}",
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
    let database = client.database("db_prices");
    info!("📊 Using database: db_prices");

    // Phase 1.3: Price Collection & Index Setup Logging
    info!("📦 Setting up prices collection...");
    let price_coll: Collection<Offer> = database.collection("prices");
    let indexes = vec![
        // Primary index for SKU queries (most selective)
        IndexModel::builder().keys(doc! { "sku": 1 }).build(),
        // Compound index for SKU + date range queries (main query pattern)
        IndexModel::builder()
            .keys(doc! { "sku": 1, "start_date": 1, "end_date": 1 })
            .build(),
        // Index for quantity range queries
        IndexModel::builder()
            .keys(doc! { "min_quantity": 1, "max_quantity": 1 })
            .build(),
        // Index for currency filtering within offer_prices array
        IndexModel::builder()
            .keys(doc! { "offer_prices.currency": 1 })
            .build(),
        // Compound index for price sorting within currency
        IndexModel::builder()
            .keys(doc! { "offer_prices.currency": 1, "offer_prices.price": 1 })
            .build(),
    ];
    info!("🔍 Creating {} price indexes...", indexes.len());
    match price_coll.create_indexes(indexes).await {
        Ok(result) => {
            info!(
                "✅ Created {} price indexes successfully",
                result.index_names.len()
            );
            debug!("Price indexes: sku, sku+dates, quantity_ranges, currency, currency+price");
        }
        Err(e) => {
            error!("❌ Failed to create price indexes: {e}");
            return Err(e.into());
        }
    }

    // Phase 2.1: DAO Setup Logging
    info!("🏗️  Initializing data access objects...");
    let offer_dao = Arc::new(OfferDaoImpl::new(price_coll));
    debug!("✅ Offer DAO initialized");

    // Phase 2.2: Router Setup Logging
    info!("🛣️  Setting up message router...");
    let mut router = Router::new();
    router
        .add_route(
            "create_offer".to_owned(),
            Box::new(|d, m| Box::pin(create_offer(d, m))),
        )
        .add_route(
            "get_offer".to_owned(),
            Box::new(|d, m| Box::pin(get_offer(d, m))),
        )
        .add_route(
            "delete_offer".to_owned(),
            Box::new(|d, m| Box::pin(delete_offer(d, m))),
        )
        .add_route(
            "get_best_offer_price".to_owned(),
            Box::new(|d, m| Box::pin(get_best_offer_price(d, m))),
        )
        .add_route(
            "get_best_offer_prices".to_owned(),
            Box::new(|d, m| Box::pin(get_best_offer_prices(d, m))),
        );

    let route_count = 5;
    info!("✅ Configured {route_count} price routes");
    debug!("Price routes: create_offer, get_offer, delete_offer, get_best_offer_price, get_best_offer_prices");

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
    validate_price_dependencies(&client, &nats_client).await?;

    // Phase 3.1: Queue Subscription Logging
    info!("📡 Subscribing to NATS queue: offers.*");
    let requests = match nats_client
        .queue_subscribe("offers.*", "queue".to_owned())
        .await
    {
        Ok(subscription) => {
            info!("✅ Successfully subscribed to offers.* queue");
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

    info!("🚀 Price service is ready and listening for requests");
    info!("📊 Service startup completed successfully");

    // Phase 3.2: Request Processing Logging
    requests
        .for_each_concurrent(25, |request| {
            let od = offer_dao.clone();
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
                    "📨 Processing price operation: {} from subject: {}",
                    operation, request.subject
                );

                // Phase 5: Use OperationTimer for performance monitoring
                let _timer = OperationTimer::new(&format!("offers.{operation}"));

                let result = if let Some(handler) = routes.get(&operation) {
                    // Note: Price service handlers return Response objects that need to be published
                    let response = handler.call(od, request).await;
                    // Publish response manually here since we don't have the Router::route method integrated
                    if let Err(e) = client_clone
                        .publish(response.subject, response.payload)
                        .await
                    {
                        error!("❌ Failed to publish response: {e}");
                        Err(Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
                    } else {
                        debug!("✅ Price operation {operation} completed");
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
