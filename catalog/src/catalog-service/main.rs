mod handlers;
mod model;
mod persistence;

use bson::doc;
use handlers::{
    create_product, delete_product, get_product, get_product_by_slug, search_products, export_products, update_product, get_product_slugs, Router,
    category_service::CategoryService,
    category_handlers::{handle_create_category, handle_get_category, handle_get_category_by_slug, handle_export_categories, handle_update_category, handle_delete_category, handle_import_categories, handle_get_category_tree},
};
use persistence::{
    product_dao::ProductDaoImpl,
    category_dao::CategoryDaoImpl,
};
use std::{env, error::Error, sync::Arc};

use dotenvy::dotenv;
use log::{debug, error, info};

use futures::StreamExt;
use model::{Product, Category, CategoryTreeCache};
use mongodb::{Client, Collection, IndexModel};

pub mod catalog_messages {
    include!(concat!(env!("OUT_DIR"), "/catalog_messages.rs"));
}

#[derive(Clone)]
pub struct AppState {
    pub product_dao: Arc<dyn persistence::product_dao::ProductDao + Send + Sync>,
    pub category_service: Arc<CategoryService>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    pretty_env_logger::init();
    info!("Starting catalog service");

    // initialize dotenv
    dotenv().ok();

    // Get MongoDB URL
    let uri = env::var("MONGODB_URL").expect("MONGODB_URL must be set");
    // connect to MongoDB
    let client = Client::with_uri_str(uri).await?;
    let database = client.database("db_catalog");
    
    // Product collection setup
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
    products_coll.create_indexes(indexes).await?;

    // Category collection setup
    let categories_coll: Collection<Category> = database.collection("categories");
    let category_cache_coll: Collection<CategoryTreeCache> = database.collection("category_tree_cache");
    
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
        IndexModel::builder()
            .keys(doc! { "path": 1 })
            .build(),
        IndexModel::builder()
            .keys(doc! { "parent_id": 1 })
            .build(),
        IndexModel::builder()
            .keys(doc! { "ancestors": 1 })
            .build(),
        IndexModel::builder()
            .keys(doc! { "level": 1 })
            .build(),
        IndexModel::builder()
            .keys(doc! { "is_active": 1, "display_order": 1 })
            .build(),
    ];
    categories_coll.create_indexes(category_indexes).await?;

    let product_dao = Arc::new(ProductDaoImpl::new(products_coll, database.clone()));
    let category_dao = Arc::new(CategoryDaoImpl::new(categories_coll, category_cache_coll));
    let category_service = Arc::new(CategoryService::new(category_dao));

    let app_state = AppState { 
        product_dao,
        category_service,
    };

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

    // Connect to the nats server
    let nats_client = async_nats::connect("0.0.0.0:4222").await?;

    let requests = nats_client
        .queue_subscribe("catalog.*", "queue".to_owned())
        .await?;

    let routes = Arc::new(router.route_map);

    info!("Catalog service listening on catalog.* queue");
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
                debug!("Processing catalog operation: {}", operation);

                // Handle category operations separately
                let result = match operation.as_str() {
                    "create_category" => {
                        let response = handle_create_category(&request, cs).await;
                        match response {
                            Ok(response_bytes) => {
                                if let Some(reply) = request.reply {
                                    let _ = client_clone.publish(reply, response_bytes.into()).await;
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
                                    let _ = client_clone.publish(reply, response_bytes.into()).await;
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
                                    let _ = client_clone.publish(reply, response_bytes.into()).await;
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
                                    let _ = client_clone.publish(reply, response_bytes.into()).await;
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
                                    let _ = client_clone.publish(reply, response_bytes.into()).await;
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
                                    let _ = client_clone.publish(reply, response_bytes.into()).await;
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
                                    let _ = client_clone.publish(reply, response_bytes.into()).await;
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
                                    let _ = client_clone.publish(reply, response_bytes.into()).await;
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
                            error!("No handler found for operation: {}", operation);
                            Ok(())
                        }
                    }
                };

                match result {
                    Ok(_) => debug!("Successfully processed {}", operation),
                    Err(e) => error!("Error processing {}: {:?}", operation, e),
                }
            }
        })
        .await;

    Ok(())
}
