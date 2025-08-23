use crate::{
    domain::{Category, CategoryTreeCache, Product},
    handlers::{
        category_handlers::{
            create_category, delete_category, export_categories, get_category,
            get_category_by_slug, get_category_tree, import_categories, update_category,
        },
        product_handlers::{
            create_product, delete_product, export_products, get_product, get_product_by_slug,
            get_product_slugs, search_products, update_product,
        },
        Router,
    },
    persistence::{category_dao::CategoryDaoImpl, product_dao::ProductDaoImpl},
    services::{category_service::CategoryService, product_service::ProductService},
    AppState,
};

use async_nats::Client as NatsClient;
use bson::doc;
use futures::StreamExt;
use log::{debug, error, info};
use mongodb::{Client as MongoClient, Collection, Database, IndexModel};
use rust_common::OperationTimer;
use std::{env, error::Error, sync::Arc};

pub struct Application {
    pub nats_client: NatsClient,
    pub mongodb_client: MongoClient,
    pub database: Database,
    pub app_state: AppState,
    routes: Arc<std::collections::HashMap<String, handlers::RouteHandler>>,
}

pub struct Settings {
    pub mongodb_url: String,
    pub nats_url: String,
    pub database_name: String,
}

impl Settings {
    pub fn from_env() -> Self {
        Self {
            mongodb_url: env::var("MONGODB_URL").expect("MONGODB_URL must be set"),
            nats_url: env::var("NATS_URL").unwrap_or_else(|_| "nats://localhost:4222".to_string()),
            database_name: env::var("CATALOG_DB_NAME").unwrap_or_else(|_| "db_catalog".to_string()),
        }
    }

    pub fn for_test(database_name: String) -> Self {
        Self {
            mongodb_url: env::var("MONGODB_URL")
                .unwrap_or_else(|_| "mongodb://admin:changeme@localhost:27017".to_string()),
            nats_url: env::var("NATS_URL").unwrap_or_else(|_| "nats://localhost:4222".to_string()),
            database_name,
        }
    }
}

impl Application {
    pub async fn build(settings: Settings) -> Result<Self, Box<dyn Error + Send + Sync>> {
        // Connect to MongoDB
        info!("🔗 Connecting to MongoDB...");
        let mongodb_client = MongoClient::with_uri_str(&settings.mongodb_url).await?;
        info!("✅ Successfully connected to MongoDB");

        // Test the connection
        mongodb_client.list_database_names().await?;

        let database = mongodb_client.database(&settings.database_name);
        info!("📊 Using database: {}", settings.database_name);

        // Setup collections and indexes
        let products_coll = Self::setup_products_collection(&database).await?;
        let (categories_coll, category_cache_coll) =
            Self::setup_categories_collections(&database).await?;

        // Initialize DAOs
        let product_dao = Arc::new(ProductDaoImpl::new(products_coll, database.clone()));
        let category_dao = Arc::new(CategoryDaoImpl::new(categories_coll, category_cache_coll));

        // Initialize services
        let product_service = Arc::new(ProductService::new(product_dao.clone()));
        let category_service = Arc::new(CategoryService::new(category_dao.clone()));

        let app_state = AppState {
            product_dao,
            category_dao,
            product_service,
            category_service,
        };

        // Setup router
        let routes = Self::setup_routes();

        // Connect to NATS
        info!("🔗 Connecting to NATS server: {}", settings.nats_url);
        let nats_client = async_nats::connect(&settings.nats_url).await?;
        info!("✅ Successfully connected to NATS");

        Ok(Self {
            nats_client,
            mongodb_client,
            database,
            app_state,
            routes,
        })
    }

    pub async fn setup_products_collection(
        database: &Database,
    ) -> Result<Collection<Product>, Box<dyn Error + Send + Sync>> {
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
        let result = products_coll.create_indexes(indexes).await?;
        info!(
            "✅ Created {} product indexes successfully",
            result.index_names.len()
        );

        Ok(products_coll)
    }

    pub async fn setup_categories_collections(
        database: &Database,
    ) -> Result<(Collection<Category>, Collection<CategoryTreeCache>), Box<dyn Error + Send + Sync>>
    {
        info!("📁 Setting up categories collection...");
        let categories_coll: Collection<Category> = database.collection("categories");
        let category_cache_coll: Collection<CategoryTreeCache> =
            database.collection("category_tree_cache");

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
        let result = categories_coll.create_indexes(category_indexes).await?;
        info!(
            "✅ Created {} category indexes successfully",
            result.index_names.len()
        );

        Ok((categories_coll, category_cache_coll))
    }

    fn setup_routes() -> Arc<std::collections::HashMap<String, handlers::RouteHandler>> {
        info!("🛣️  Setting up message router from proto definitions...");
        let mut router = Router::new();

        // Product routes from generated configuration
        for (method, _subject) in crate::nats_config::product::routes() {
            match method {
                "create_product" => router.add_route(
                    method.to_owned(),
                    Box::new(|d, c, m| Box::pin(create_product(d, c, m))),
                ),
                "get_product" => router.add_route(
                    method.to_owned(),
                    Box::new(|d, c, m| Box::pin(get_product(d, c, m))),
                ),
                "get_product_by_slug" => router.add_route(
                    method.to_owned(),
                    Box::new(|d, c, m| Box::pin(get_product_by_slug(d, c, m))),
                ),
                "update_product" => router.add_route(
                    method.to_owned(),
                    Box::new(|d, c, m| Box::pin(update_product(d, c, m))),
                ),
                "delete_product" => router.add_route(
                    method.to_owned(),
                    Box::new(|d, c, m| Box::pin(delete_product(d, c, m))),
                ),
                "search_products" => router.add_route(
                    method.to_owned(),
                    Box::new(|d, c, m| Box::pin(search_products(d, c, m))),
                ),
                "export_products" => router.add_route(
                    method.to_owned(),
                    Box::new(|d, c, m| Box::pin(export_products(d, c, m))),
                ),
                "get_product_slugs" => router.add_route(
                    method.to_owned(),
                    Box::new(|d, c, m| Box::pin(get_product_slugs(d, c, m))),
                ),
                _ => &mut router,
            };
        }

        // Category routes from generated configuration
        for (method, _subject) in crate::nats_config::category::routes() {
            match method {
                "create_category" => router.add_route(
                    method.to_owned(),
                    Box::new(|d, c, m| Box::pin(create_category(d, c, m))),
                ),
                "get_category" => router.add_route(
                    method.to_owned(),
                    Box::new(|d, c, m| Box::pin(get_category(d, c, m))),
                ),
                "get_category_by_slug" => router.add_route(
                    method.to_owned(),
                    Box::new(|d, c, m| Box::pin(get_category_by_slug(d, c, m))),
                ),
                "update_category" => router.add_route(
                    method.to_owned(),
                    Box::new(|d, c, m| Box::pin(update_category(d, c, m))),
                ),
                "delete_category" => router.add_route(
                    method.to_owned(),
                    Box::new(|d, c, m| Box::pin(delete_category(d, c, m))),
                ),
                "export_categories" => router.add_route(
                    method.to_owned(),
                    Box::new(|d, c, m| Box::pin(export_categories(d, c, m))),
                ),
                "import_categories" => router.add_route(
                    method.to_owned(),
                    Box::new(|d, c, m| Box::pin(import_categories(d, c, m))),
                ),
                "get_category_tree" => router.add_route(
                    method.to_owned(),
                    Box::new(|d, c, m| Box::pin(get_category_tree(d, c, m))),
                ),
                // TODO: Implement these handlers
                // "get_children" => router.add_route(
                //     method.to_owned(),
                //     Box::new(|d, c, m| Box::pin(get_children(d, c, m))),
                // ),
                // "get_descendants" => router.add_route(
                //     method.to_owned(),
                //     Box::new(|d, c, m| Box::pin(get_descendants(d, c, m))),
                // ),
                // "move_category" => router.add_route(
                //     method.to_owned(),
                //     Box::new(|d, c, m| Box::pin(move_category(d, c, m))),
                // ),
                // "get_category_path" => router.add_route(
                //     method.to_owned(),
                //     Box::new(|d, c, m| Box::pin(get_category_path(d, c, m))),
                // ),
                // "reorder_children" => router.add_route(
                //     method.to_owned(),
                //     Box::new(|d, c, m| Box::pin(reorder_children(d, c, m))),
                // ),
                _ => &mut router,
            };
        }

        let product_count = crate::nats_config::product::routes().len();
        let category_count = crate::nats_config::category::routes().len();
        info!(
            "✅ Configured {product_count} product routes and {category_count} category routes from proto definitions"
        );
        Arc::new(router.route_map)
    }

    pub async fn run(self) -> Result<(), Box<dyn Error + Send + Sync>> {
        // Use generated configuration for NATS subscription
        let subscription_pattern = format!("{}.*", crate::nats_config::product::SUBJECT_PREFIX);
        let queue_name = crate::nats_config::product::QUEUE;

        info!("📡 Subscribing to NATS queue '{queue_name}' with pattern: {subscription_pattern}");
        let requests = self
            .nats_client
            .queue_subscribe(subscription_pattern.clone(), queue_name.to_owned())
            .await?;
        info!("✅ Successfully subscribed to {subscription_pattern} on queue '{queue_name}'");

        info!("🚀 Catalog service is ready and listening for requests");

        let routes = self.routes.clone();
        let app_state = self.app_state.clone();
        let nats_client = self.nats_client.clone();

        // Publish event to NATS that the application has started using the database name
        // for the subject with the message "ApplicationStarted"
        let subject = format!("application.{}.events", self.database.name());
        nats_client
            .publish(subject.clone(), "ApplicationStarted".as_bytes().into())
            .await?;
        info!(
            "📣 Published 'ApplicationStarted' event to NATS on subject: {}",
            &subject
        );

        requests
            .for_each_concurrent(25, |request| {
                let app_state = Arc::new(app_state.clone());
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

                    let _timer = OperationTimer::new(&format!("catalog.{operation}"));

                    // Route all operations through the router
                    let result = if let Some(handler) = routes.get(&operation) {
                        handler(app_state, client_clone, request).await
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
}

use crate::handlers;
