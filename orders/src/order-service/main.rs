pub mod model;

use dotenvy::dotenv;
use handlers::{create_order, delete_order, get_order, Router};
// use handlers::create_order;
// use handlers::{create_order, delete_order, get_order};
use log::{debug, error, info};
use persistence::orders_dao::{OrdersDao, OrdersDaoImpl};
use std::{borrow::Borrow, env, error::Error, sync::Arc};
use tokio::signal;

use model::Order;
use mongodb::{Client, Collection};

use futures::StreamExt;

pub mod order_messages {
    include!(concat!(env!("OUT_DIR"), "/order_messages.rs"));
}

mod handlers;
mod persistence;

#[derive(Clone)]
pub struct AppState {
    pub orders_dao: Arc<dyn OrdersDao + Send + Sync>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    // let dir = env!("OUT_DIR");
    // initialize pretty_env_logger
    pretty_env_logger::init();
    // info!("Test info logging");
    // warn!("Test warn logging");
    // error!("Test error loggin: {}", dir);
    // trace!("test trace logging");
    // debug!("test debug logging");

    // initialize dotenv
    dotenv().ok();

    // Get MongoDB URL
    let uri = env::var("MONGODB_URL").expect("MONGODB_URL must be set");
    // connect to MongoDB
    let client = Client::with_uri_str(uri).await?;
    let database = client.database("db_orders");
    let orders_coll: Collection<Order> = database.collection("orders");

    let orders_dao = Arc::new(OrdersDaoImpl::new(orders_coll)).clone();

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

    // Connect to the nats server
    let client = async_nats::connect("0.0.0.0:4222").await?;

    let requests = client
        .queue_subscribe("orders.*", "queue".to_owned())
        .await?;

    let routes = router.route_map.borrow();

    debug!("Inside spawn order create service");
    requests
        .for_each_concurrent(25, |request| {
            // while let Some(request) = create_order.next().await {
            let od = orders_dao.clone();
            let client = client.clone();
            async move {
                // Take everything after "orders.*"
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
                                "Missing route after orders. example: orders.create_order".into(),
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

// async fn main() -> Result<(), Box<dyn Error>> {
//     // initialize pretty_env_logger
//     pretty_env_logger::init();
//     info!("Test info logging");
//     warn!("Test warn logging");
//     error!("Test error logging");
//     trace!("test trace logging");
//     debug!("test debug logging");

//     // initialize dotenv
//     dotenv().ok();

//     // Get MongoDB URL
//     let uri = env::var("MONGODB_URL").expect("MONGODB_URL must be set");
//     // connect to MongoDB
//     let client = Client::with_uri_str(uri).await?;
//     let database = client.database("db_orders");
//     let orders_coll: Collection<Order> = database.collection("orders");

//     let orders_dao = Arc::new(OrdersDaoImpl::new(orders_coll)).clone();

//     let app_state = AppState { orders_dao };

//     let app = Router::new()
//         .route(
//             "/hello",
//             get(|| async { Html("hello <strong>World!!</strong>") }),
//         )
//         .route("/order", post(create_order))
//         .route("/order/:id", get(get_order))
//         .route("/order", delete(delete_order))
//         .with_state(app_state);

//     // Start Server
//     let listener = tokio::net::TcpListener::bind("127.0.0.1:8080")
//         .await
//         .unwrap();
//     println!("->> LISTENING on {:?}\n", listener);
//     axum::serve(listener, app).await.unwrap();
//     Ok(())
// }
