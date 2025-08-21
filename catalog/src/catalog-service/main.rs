mod handlers;
mod model;
mod persistence;

use bson::doc;
use handlers::{
    category_handlers::{
        handle_create_category, handle_delete_category, handle_export_categories,
        handle_get_category, handle_get_category_by_slug, handle_get_category_tree,
        handle_import_categories, handle_update_category,
    },
    category_service::CategoryService,
    create_product, delete_product, export_products, get_product, get_product_by_slug,
    get_product_slugs, search_products, update_product, Router,
};
use persistence::{category_dao::CategoryDaoImpl, product_dao::ProductDaoImpl};
use std::{
    env,
    error::Error,
    io::{self, Write},
    sync::Arc,
};

use log::{debug, error, info};
use rust_common::{
    load_environment, mask_sensitive_url, setup_signal_handlers, validate_catalog_dependencies,
    HealthMonitor, OperationTimer,
};

use futures::StreamExt;
use model::{Category, CategoryTreeCache, Product};
use mongodb::{Client, Collection, IndexModel};

// Import common module for generated proto code
mod common {
    pub use shared_proto::common::*;
}

pub mod catalog_messages {
    include!(concat!(env!("OUT_DIR"), "/catalog_messages.rs"));

    // Re-export common types for backward compatibility
    pub use super::common::{Code, Status};
}

#[derive(Clone)]
pub struct AppState {
    pub product_dao: Arc<dyn persistence::product_dao::ProductDao + Send + Sync>,
    pub category_service: Arc<CategoryService>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    // Early boot diagnostic
    println!("BOOT: entering main");
    let _ = io::stdout().flush();

    // Load environment configuration FIRST, before initializing logger
    load_environment();

    // Initialize logger after loading environment (so RUST_LOG from .env is used)
    pretty_env_logger::init();

    // Phase 1.1: Environment & Configuration Logging
    info!(
        "🚀 Starting Rust Commerce Catalog Service v{}",
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
    let database = client.database("db_catalog");
    info!("📊 Using database: db_catalog");

    // Phase 1.3: Product Collection & Index Setup Logging
    info!("📦 Setting up products collection...");
    let products_coll: Collection<Product> = database.collection("products");
    let indexes = vec![
        IndexModel::builder()
            .keys(doc! { "product_ref": 1})
            .options(
                mongodb::options::IndexOptions::builder()
                    .unique(true)
                    .build(),
            )
            .build(),
        IndexModel::builder()
            .keys(doc! { "slug": 1 })
            .options(
                mongodb::options::IndexOptions::builder()
                    .unique(true)
                    .build(),
            )
            .build(),
    ];
    info!("🔍 Creating {} product indexes...", indexes.len());
    match products_coll.create_indexes(indexes).await {
        Ok(result) => {
            info!(
                "✅ Created {} product indexes successfully",
                result.index_names.len()
            );
            debug!("Product indexes: product_ref (unique), slug (unique)");
        }
        Err(e) => {
            error!("❌ Failed to create product indexes: {e}");
            return Err(e.into());
        }
    }

    // Category collection setup
    info!("📁 Setting up categories collection...");
    let categories_coll: Collection<Category> = database.collection("categories");
    let category_cache_coll: Collection<CategoryTreeCache> =
        database.collection("category_tree_cache");

    // Create category indexes
    let category_indexes = vec![
        IndexModel::builder()
            .keys(doc! { "slug": 1 })
            .options(
                mongodb::options::IndexOptions::builder()
                    .unique(true)
                    .build(),
            )
            .build(),
        IndexModel::builder().keys(doc! { "path": 1 }).build(),
        IndexModel::builder().keys(doc! { "parent_id": 1 }).build(),
        IndexModel::builder().keys(doc! { "ancestors": 1 }).build(),
        IndexModel::builder().keys(doc! { "level": 1 }).build(),
        IndexModel::builder()
            .keys(doc! { "is_active": 1, "display_order": 1 })
            .build(),
    ];
    info!("🔍 Creating {} category indexes...", category_indexes.len());
    match categories_coll.create_indexes(category_indexes).await {
        Ok(result) => {
            info!(
                "✅ Created {} category indexes successfully",
                result.index_names.len()
            );
            debug!("Category indexes: slug (unique), path, parent_id, ancestors, level, is_active+display_order");
        }
        Err(e) => {
            error!("❌ Failed to create category indexes: {e}");
            return Err(e.into());
        }
    }

    // Phase 2.1: DAO & Service Setup Logging
    info!("🏗️  Initializing data access objects...");
    let product_dao = Arc::new(ProductDaoImpl::new(products_coll, database.clone()));
    debug!("✅ Product DAO initialized");

    let category_dao = Arc::new(CategoryDaoImpl::new(categories_coll, category_cache_coll));
    debug!("✅ Category DAO initialized");

    let category_service = Arc::new(CategoryService::new(category_dao));
    debug!("✅ Category Service initialized");

    let app_state = AppState {
        product_dao,
        category_service,
    };
    debug!("✅ Application state initialized");

    // Phase 2.2: Router Setup Logging
    info!("🛣️  Setting up message router...");
    let mut router = Router::new();
    router
        .add_route(
            "create_product".to_owned(),
            Box::new(|d, c, m| Box::pin(create_product(d, c, m))),
        )
        .add_route(
            "get_product".to_owned(),
            Box::new(|d, c, m| Box::pin(get_product(d, c, m))),
        )
        .add_route(
            "get_product_by_slug".to_owned(),
            Box::new(|d, c, m| Box::pin(get_product_by_slug(d, c, m))),
        )
        .add_route(
            "update_product".to_owned(),
            Box::new(|d, c, m| Box::pin(update_product(d, c, m))),
        )
        .add_route(
            "delete_product".to_owned(),
            Box::new(|d, c, m| Box::pin(delete_product(d, c, m))),
        )
        .add_route(
            "search_products".to_owned(),
            Box::new(|d, c, m| Box::pin(search_products(d, c, m))),
        )
        .add_route(
            "export_products".to_owned(),
            Box::new(|d, c, m| Box::pin(export_products(d, c, m))),
        )
        .add_route(
            "get_product_slugs".to_owned(),
            Box::new(|d, c, m| Box::pin(get_product_slugs(d, c, m))),
        );

    let route_count = 8; // Product routes (categories handled separately)
    info!("✅ Configured {route_count} product routes");
    debug!("Product routes: create_product, get_product, get_product_by_slug, update_product, delete_product, search_products, export_products, get_product_slugs");
    debug!("Category routes handled separately: create_category, get_category, get_category_by_slug, export_categories, update_category, delete_category, import_categories, get_category_tree");

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
    validate_catalog_dependencies(&client, &nats_client).await?;

    // Phase 3.1: Queue Subscription Logging
    info!("📡 Subscribing to NATS queue: catalog.*");
    let requests = match nats_client
        .queue_subscribe("catalog.*", "queue".to_owned())
        .await
    {
        Ok(subscription) => {
            info!("✅ Successfully subscribed to catalog.* queue");
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

    info!("🚀 Catalog service is ready and listening for requests");
    info!("📊 Service startup completed successfully");

    // Phase 3.2: Request Processing Logging
    requests
        .for_each_concurrent(25, |request| {
            let pd = app_state.product_dao.clone();
            let cs = app_state.category_service.clone();
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
                    "📨 Processing catalog operation: {} from subject: {}",
                    operation, request.subject
                );

                // Phase 5: Use OperationTimer for performance monitoring
                let _timer = OperationTimer::new(&format!("catalog.{operation}"));

                // Handle category operations separately
                let result = match operation.as_str() {
                    "create_category" => {
                        let response = handle_create_category(&request, cs).await;
                        match response {
                            Ok(response_bytes) => {
                                if let Some(reply) = request.reply {
                                    let _ =
                                        client_clone.publish(reply, response_bytes.into()).await;
                                }
                                Ok(())
                            }
                            Err(e) => Err(e),
                        }
                    }
                    "get_category" => {
                        let response = handle_get_category(&request, cs).await;
                        match response {
                            Ok(response_bytes) => {
                                if let Some(reply) = request.reply {
                                    let _ =
                                        client_clone.publish(reply, response_bytes.into()).await;
                                }
                                Ok(())
                            }
                            Err(e) => Err(e),
                        }
                    }
                    "get_category_by_slug" => {
                        let response = handle_get_category_by_slug(&request, cs).await;
                        match response {
                            Ok(response_bytes) => {
                                if let Some(reply) = request.reply {
                                    let _ =
                                        client_clone.publish(reply, response_bytes.into()).await;
                                }
                                Ok(())
                            }
                            Err(e) => Err(e),
                        }
                    }
                    "export_categories" => {
                        let response = handle_export_categories(&request, cs).await;
                        match response {
                            Ok(response_bytes) => {
                                if let Some(reply) = request.reply {
                                    let _ =
                                        client_clone.publish(reply, response_bytes.into()).await;
                                }
                                Ok(())
                            }
                            Err(e) => Err(e),
                        }
                    }
                    "update_category" => {
                        let response = handle_update_category(&request, cs).await;
                        match response {
                            Ok(response_bytes) => {
                                if let Some(reply) = request.reply {
                                    let _ =
                                        client_clone.publish(reply, response_bytes.into()).await;
                                }
                                Ok(())
                            }
                            Err(e) => Err(e),
                        }
                    }
                    "delete_category" => {
                        let response = handle_delete_category(&request, cs).await;
                        match response {
                            Ok(response_bytes) => {
                                if let Some(reply) = request.reply {
                                    let _ =
                                        client_clone.publish(reply, response_bytes.into()).await;
                                }
                                Ok(())
                            }
                            Err(e) => Err(e),
                        }
                    }
                    "import_categories" => {
                        let response = handle_import_categories(&request, cs).await;
                        match response {
                            Ok(response_bytes) => {
                                if let Some(reply) = request.reply {
                                    let _ =
                                        client_clone.publish(reply, response_bytes.into()).await;
                                }
                                Ok(())
                            }
                            Err(e) => Err(e),
                        }
                    }
                    "get_category_tree" => {
                        let response = handle_get_category_tree(&request, cs).await;
                        match response {
                            Ok(response_bytes) => {
                                if let Some(reply) = request.reply {
                                    let _ =
                                        client_clone.publish(reply, response_bytes.into()).await;
                                }
                                Ok(())
                            }
                            Err(e) => Err(e),
                        }
                    }
                    _ => {
                        // Handle product operations through existing router
                        if let Some(handler) = routes.get(&operation) {
                            handler(pd, client_clone, request).await
                        } else {
                            error!("No handler found for operation: {operation}");
                            Ok(())
                        }
                    }
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
