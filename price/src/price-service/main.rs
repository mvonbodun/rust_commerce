mod handlers;
mod model;
mod persistence;

use handlers::{create_offer, delete_offer, get_offer, get_best_offer_price, get_best_offer_prices, Router};
use persistence::offer_dao::OfferDaoImpl;
use std::{borrow::Borrow, env, error::Error, sync::Arc};
use tokio::signal;

use dotenvy::dotenv;
use log::{debug, error, info};

use futures::StreamExt;
use model::Offer;
use mongodb::{Client, Collection};

pub mod offer_messages {
    include!(concat!(env!("OUT_DIR"), "/offer_messages.rs"));
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
    let database = client.database("db_prices");
    let price_coll: Collection<Offer> = database.collection("prices");

    let offer_dao = Arc::new(OfferDaoImpl::new(price_coll)).clone();

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

    // Connect to the nats server
    let client = async_nats::connect("0.0.0.0:4222").await?;

    let requests = client
        .queue_subscribe("offers.*", "queue".to_owned())
        .await?;

    let routes = router.route_map.borrow();

    requests
        .for_each_concurrent(25, |request| {
            let od = offer_dao.clone();
            let client = client.clone();
            async move {
                // Take everything after "offers.*"
                // let route = &request.subject[7..];
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
                                "Missing route after offers. example: offers.create_offer".into(),
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
