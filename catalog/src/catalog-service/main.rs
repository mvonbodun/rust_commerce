mod handlers;
mod model;
mod persistence;

use handlers::{create_product, delete_product, get_product, search_products, update_product, Router};
use persistence::product_dao::ProductDaoImpl;
use std::{env, error::Error, sync::Arc};

use dotenvy::dotenv;
use log::{debug, error, info};

use futures::StreamExt;
use model::Product;
use mongodb::{Client, Collection};

pub mod catalog_messages {
    include!(concat!(env!("OUT_DIR"), "/catalog_messages.rs"));
}

#[derive(Clone)]
pub struct AppState {
    pub product_dao: Arc<dyn persistence::product_dao::ProductDao + Send + Sync>,
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
    let products_coll: Collection<Product> = database.collection("products");

    let product_dao = Arc::new(ProductDaoImpl::new(products_coll)).clone();

    let app_state = AppState { product_dao };

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

                if let Some(handler) = routes.get(&operation) {
                    let result = handler(pd, client_clone, request).await;
                    match result {
                        Ok(_) => debug!("Successfully processed {}", operation),
                        Err(e) => error!("Error processing {}: {:?}", operation, e),
                    }
                } else {
                    error!("No handler found for operation: {}", operation);
                }
            }
        })
        .await;

    Ok(())
}
