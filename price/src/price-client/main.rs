use offer_messages::{
    OfferCreateRequest, OfferCreateResponse, OfferDeleteRequest, OfferDeleteResponse,
    OfferGetRequest, OfferGetResponse,
};
use prost::Message;
use prost_types::Timestamp;
use uuid::Uuid;

pub mod offer_messages {
    include!(concat!(env!("OUT_DIR"), "/offer_messages.rs"));
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = async_nats::connect("0.0.0.0:4222").await?;

    // Create an offer
    let offer = OfferCreateRequest {
        item_id: Uuid::new_v4().to_string(),
        item_ref: Some("item_ref1".to_string()),
        start_date: Some(Timestamp {
            seconds: 1629459200,
            nanos: 0,
        }),
        end_date: Some(Timestamp {
            seconds: 1632055200,
            nanos: 0,
        }),
        min_quantity: 1,
        max_quantity: Some(500),
        offer_prices: vec![offer_messages::OfferPrice {
            price: "50.74".to_string(),
            currency: "USD".to_string(),
        }],
    };

    let mut buf = vec![];
    offer.encode(&mut buf)?;
    let result = client.request("offers.create_offer", buf.into()).await?;
    let response = OfferCreateResponse::decode(result.payload)?;
    println!("response: {:?}", response);

    let offer_id = response
        .offer
        .expect("failed to create offer")
        .id
        .expect("failed to get id from offer");
    // Get Offer
    let offer_get_request = OfferGetRequest {
        id: offer_id.clone(),
    };

    let mut buf = vec![];
    offer_get_request.encode(&mut buf)?;
    let result = client.request("offers.get_offer", buf.into()).await?;
    let response = OfferGetResponse::decode(result.payload)?;
    println!("response from get_offer: {:?}", response);

    let offer_delete_request = OfferDeleteRequest {
        id: offer_id.clone(),
    };

    // let mut buf = vec![];
    // offer_delete_request.encode(&mut buf)?;
    // let result = client.request("offers.delete_offer", buf.into()).await?;
    // let response = OfferDeleteResponse::decode(result.payload)?;
    // println!("response from delete_offer: {:?}", response);

    Ok(())
}
