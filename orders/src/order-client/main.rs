use clap::{Parser, Subcommand};
use order_messages::{
    Address, OrderCreateRequest, OrderCreateResponse, OrderDeleteRequest, OrderDeleteResponse,
    OrderGetRequest, OrderGetResponse,
};
use prost::Message;

use rust_common::env_config;

pub mod order_messages {
    include!(concat!(env!("OUT_DIR"), "/order_messages.rs"));
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    OrderCreate {
        #[arg(short, long)]
        order_ref: String,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables
    env_config::load_environment();
    
    // Get NATS URL from environment
    let nats_url = std::env::var("NATS_URL").unwrap_or_else(|_| "nats://localhost:4222".to_string());
    
    // Connect to the nats server
    let client = async_nats::connect(&nats_url).await?;

    // Create an order
    let order = OrderCreateRequest {
        order_ref: Some("234234".to_owned()),
        sold_to: Some(Address {
            id: "123456".to_owned(),
            customer_ref: Some("asdfasdf".to_owned()),
            name: "John Doe".to_owned(),
            address_line1: "123 Main St".to_owned(),
            address_line2: None,
            company: None,
            city: "Anytown".to_owned(),
            state_province: Some("CA".to_owned()),
            postal_code: "90210".to_owned(),
            country: "USA".to_owned(),
            telephone: "123-456-7890".to_owned(),
            email: Some("john.doe@example.com".to_owned()),
        }),
    };

    let mut buf = vec![];
    order.encode(&mut buf)?;
    let result = client.request("orders.create_order", buf.into()).await?;
    let response = OrderCreateResponse::decode(result.payload)?;
    println!("response: {:?}", response);

    // Extract the order_id
    let order_id = response
        .order
        .expect("failed to create order")
        .id
        .expect("failed to get id from order");

    // Get the created order
    let order = OrderGetRequest {
        id: order_id.clone(),
    };

    let mut buf = vec![];
    order.encode(&mut buf)?;
    let result = client.request("orders.get_order", buf.into()).await?;
    let response = OrderGetResponse::decode(result.payload)?;
    println!("response from get_order: {:?}", response);

    let order_delete_request = OrderDeleteRequest {
        id: order_id.clone(),
    };

    let mut buf = vec![];
    order_delete_request.encode(&mut buf)?;
    let result = client.request("orders.delete_order", buf.into()).await?;
    let response = OrderDeleteResponse::decode(result.payload)?;
    println!("response from delete request: {:?}", response);

    Ok(())
}
