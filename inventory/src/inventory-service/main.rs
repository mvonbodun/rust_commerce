mod handlers;
mod model;
mod persistence;

use handlers::{create_item, delete_item, get_item, update_stock, Router};
use persistence::inventory_dao::InventoryDaoImpl;
use std::{borrow::Borrow, env, error::Error, sync::Arc};
use tokio::signal;

use dotenvy::dotenv;
use log::{debug, error, info};

use futures::StreamExt;
use model::InventoryItem;
use mongodb::{Client, Collection};

pub mod inventory_messages {
    include!(concat!(env!("OUT_DIR"), "/inventory_messages.rs"));
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    pretty_env_logger::init();

    // initialize dotenv
    dotenv().ok();

    // Get MongoDB URL
    let uri = env::var("MONGODB_URL").expect("MONGODB_URL must be set");
    // connect to MongoDB
    let client = Client::with_uri_str(uri).await?;
    let database = client.database("db_inventory");
    let inventory_coll: Collection<InventoryItem> = database.collection("inventory");

    let inventory_dao = Arc::new(InventoryDaoImpl::new(inventory_coll)).clone();

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
            "delete_item".to_owned(),
            Box::new(|d, m| Box::pin(delete_item(d, m))),
        )
        .add_route(
            "update_stock".to_owned(),
            Box::new(|d, m| Box::pin(update_stock(d, m))),
        );

    // Connect to the nats server
    let client = async_nats::connect("0.0.0.0:4222").await?;

    let requests = client
        .queue_subscribe("inventory.*", "queue".to_owned())
        .await?;

    let routes = router.route_map.borrow();

    requests
        .for_each_concurrent(25, |request| {
            let od = inventory_dao.clone();
            let client = client.clone();
            async move {
                // Take everything after "inventory.*"
                let route = request.subject.split('.').nth(1);
                match route {
                    Some(r) => {
                        debug!("subject: {}   route {}", &request.subject, r);
                        Router::route(client, routes, r.to_owned(), od, request).await;
                    }
                    None => {
                        debug!("Missing route: {}", &request.subject);
                        client
                            .publish(
                                request.reply.unwrap(),
                                "Missing route after inventory. example: inventory.create_item".into(),
                            )
                            .await
                            .unwrap();
                    }
                }
            }
        })
        .await;

    match signal::ctrl_c().await {
        Ok(()) => {
            info!("Received SIGINT");
        }
        Err(err) => {
            error!("Error listening for SIGINT: {}", err);
        }
    }

    Ok(())
}
