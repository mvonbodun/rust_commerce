use std::future::Future;
use std::{collections::HashMap, sync::Arc};

use async_nats::{Client, Subject};
use bytes::Bytes;
use chrono::{DateTime, Timelike, Utc};
use futures::future::BoxFuture;
use iso_currency::Currency;
use log::{debug, error};
use prost::Message as ProstMessage;
use prost_types::Timestamp;
use bson::Decimal128;
use rust_decimal::Decimal;
use std::str::FromStr;
use uuid::Uuid;

use crate::{
    model,
    offer_messages::{self},
    persistence::offer_dao::OfferDaoImpl,
};

pub mod handlers_inner;

pub type Request = async_nats::Message;

#[derive(Clone)]
pub struct Response {
    pub subject: Subject,
    pub payload: Bytes,
}

pub trait HandlerFn {
    fn call(&self, dao: Arc<OfferDaoImpl>, req: Request) -> BoxFuture<'static, Response>;
}

impl<T, F> HandlerFn for T
where
    T: Fn(Arc<OfferDaoImpl>, Request) -> F + Sync,
    F: Future<Output = Response> + 'static + Send,
{
    fn call(&self, dao: Arc<OfferDaoImpl>, req: Request) -> BoxFuture<'static, Response> {
        Box::pin(self(dao, req))
    }
}

type RouteMap = HashMap<String, Box<dyn HandlerFn>>;
pub struct Router {
    pub route_map: RouteMap,
}

impl Router {
    pub fn new() -> Self {
        Router {
            route_map: HashMap::new(),
        }
    }
    pub fn add_route(&mut self, path: String, f: Box<dyn HandlerFn>) -> &mut Self {
        self.route_map.insert(path, f);
        self
    }
    pub async fn route(
        client: Client,
        routes: &RouteMap,
        path: String,
        offer_dao: Arc<OfferDaoImpl>,
        request: Request,
    ) {
        let r = routes.get(&path).unwrap();
        let response = r.call(offer_dao, request).await;
        client
            .publish(response.subject, response.payload)
            .await
            .unwrap();
    }
}

pub async fn create_offer(offer_dao: Arc<OfferDaoImpl>, offer_create_request: Request) -> Response {
    let offer = offer_messages::OfferCreateRequest::decode(offer_create_request.payload.clone());
    let mut offer_create_response = offer_messages::OfferCreateResponse {
        ..Default::default()
    };
    match offer {
        Ok(offer) => {
            debug!("offer: {:?}", offer);
            let model_offer = map_proto_offer_to_model_offer(offer);

            let result = handlers_inner::create_offer(model_offer, offer_dao.as_ref()).await;
            match result {
                Ok(o) => {
                    offer_create_response.offer = Some(map_model_offer_to_proto_offer(o));
                    offer_create_response.status = Some(offer_messages::Status {
                        code: offer_messages::Code::Ok.into(),
                        message: "".to_owned(),
                        details: vec![],
                    });
                }
                Err(err) => {
                    match err {
                        handlers_inner::HandlerError::InternalError(msg) => {
                            // Handle internal error
                            error!("Error creating order: {}", msg);
                            offer_create_response.status = Some(offer_messages::Status {
                                code: offer_messages::Code::Internal.into(),
                                message: format!("Error creating offer: {}", msg),
                                details: vec![],
                            });
                        }
                    }
                }
            }
        }
        Err(err) => {
            // Handle decoding error
            error!("Error decoding offer create request: {}", err);
            offer_create_response.status = Some(offer_messages::Status {
                code: offer_messages::Code::InvalidArgument.into(),
                message: format!("Invalid OfferCreateRequest format: {:?}", err),
                details: vec![],
            });
        }
    }
    let mut buf = vec![];
    offer_create_response.encode(&mut buf).unwrap();
    Response {
        subject: offer_create_request.reply.unwrap(),
        payload: buf.into(),
    }
}

pub async fn get_offer(offer_dao: Arc<OfferDaoImpl>, offer_get_request: Request) -> Response {
    let request = offer_messages::OfferGetRequest::decode(offer_get_request.payload.clone());
    let mut offer_get_response = offer_messages::OfferGetResponse {
        offer: None,
        status: Some(offer_messages::Status {
            code: offer_messages::Code::NotFound.into(),
            message: "Offer not found".to_string(),
            details: vec![],
        }),
    };
    match request {
        Ok(request) => {
            let result = handlers_inner::get_offer(request.id.clone(), offer_dao.as_ref()).await;
            match result {
                Ok(Some(o)) => {
                    offer_get_response.offer = Some(map_model_offer_to_proto_offer(o));
                    offer_get_response.status = Some(offer_messages::Status {
                        code: offer_messages::Code::Ok.into(),
                        message: "".to_owned(),
                        details: vec![],
                    });
                }
                Ok(None) => {
                    offer_get_response.offer = None;
                    offer_get_response.status = Some(offer_messages::Status {
                        code: offer_messages::Code::NotFound.into(),
                        message: "Offer not found".to_string(),
                        details: vec![],
                    });
                }
                Err(err) => {
                    // Handle internal error
                    match err {
                        handlers_inner::HandlerError::InternalError(msg) => {
                            error!("Error getting offer: {}", msg);
                            offer_get_response.status = Some(offer_messages::Status {
                                code: offer_messages::Code::Internal.into(),
                                message: format!("Error getting offer: {}", msg),
                                details: vec![],
                            });
                        }
                    }
                }
            }
        }
        Err(err) => {
            // Handle error
            error!("Error decoding offer: {}", err);
            offer_get_response.status = Some(offer_messages::Status {
                code: offer_messages::Code::InvalidArgument.into(),
                message: format!("Invalid OfferGetRequest format: {:?}", err),
                details: vec![],
            });
        }
    }
    let mut buf = vec![];
    offer_get_response.encode(&mut buf).unwrap();
    Response {
        subject: offer_get_request.reply.unwrap(),
        payload: buf.into(),
    }
}

pub async fn delete_offer(offer_dao: Arc<OfferDaoImpl>, offer_delete_request: Request) -> Response {
    let request = offer_messages::OfferDeleteRequest::decode(offer_delete_request.payload.clone());
    let mut offer_delete_response = offer_messages::OfferDeleteResponse { status: None };
    match request {
        Ok(request) => {
            let result = handlers_inner::delete_offer(request.id.clone(), offer_dao.as_ref()).await;
            match result {
                Ok(_) => {
                    offer_delete_response.status = Some(offer_messages::Status {
                        code: offer_messages::Code::Ok.into(),
                        message: "Offer deleted successfully".to_string(),
                        details: vec![],
                    });
                }
                Err(err) => {
                    // Handle internal error
                    match err {
                        handlers_inner::HandlerError::InternalError(msg) => {
                            error!("Error deleting offer: {}", msg);
                            offer_delete_response.status = Some(offer_messages::Status {
                                code: offer_messages::Code::Internal.into(),
                                message: format!("Failed to delete offer: {}", msg),
                                details: vec![],
                            });
                        }
                    }
                }
            }
        }
        Err(err) => {
            // Handle error
            error!("Error decoding offer: {}", err);
            offer_delete_response.status = Some(offer_messages::Status {
                code: offer_messages::Code::InvalidArgument.into(),
                message: format!("Invalid OfferDeleteRequest format: {:?}", err),
                details: vec![],
            });
        }
    }
    let mut buf = vec![];
    offer_delete_response.encode(&mut buf).unwrap();
    Response {
        subject: offer_delete_request.reply.unwrap(),
        payload: buf.into(),
    }
}

fn map_proto_offer_to_model_offer(offer: offer_messages::OfferCreateRequest) -> model::Offer {
    model::Offer {
        id: Some(Uuid::new_v4().to_string()),
        item_id: offer.item_id.clone(),
        item_ref: offer.item_ref.clone(),
        start_date: offer
            .start_date
            .unwrap()
            .to_string()
            .parse::<DateTime<Utc>>()
            .unwrap(),
        end_date: offer
            .end_date
            .unwrap()
            .to_string()
            .parse::<DateTime<Utc>>()
            .unwrap(),
        min_quantity: offer.min_quantity,
        max_quantity: offer.max_quantity,
        offer_prices: offer
            .offer_prices
            .iter()
            .map(|op| model::OfferPrice {
                price: Decimal128::from_str(&Decimal::from_f32_retain(op.price).unwrap().to_string()).unwrap(),
                currency: Currency::from_code(op.currency.as_str()).expect("currency is not valid"),
            })
            .collect(),
    }
}

// Map a model offer to a protocol buffer offer
fn map_model_offer_to_proto_offer(offer: model::Offer) -> offer_messages::Offer {
    offer_messages::Offer {
        id: offer.id,
        item_id: offer.item_id,
        item_ref: offer.item_ref,
        start_date: Some(Timestamp {
            seconds: offer.start_date.second() as i64,
            nanos: offer.start_date.nanosecond() as i32,
        }),
        end_date: Some(Timestamp {
            seconds: offer.end_date.second() as i64,
            nanos: offer.end_date.nanosecond() as i32,
        }),
        min_quantity: offer.min_quantity,
        max_quantity: offer.max_quantity,
        offer_prices: offer
            .offer_prices
            .iter()
            .map(|op| offer_messages::OfferPrice {
                price: op
                    .price
                    .to_string()
                    .parse::<f32>()
                    .expect("failed to convert decimal to f32"),
                currency: op.currency.to_string(),
            })
            .collect(),
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_map_proto_offer_to_model_offer() {
        let offer_create_request = offer_messages::OfferCreateRequest {
            item_id: "item1".to_string(),
            item_ref: Some("item_ref1".to_string()),
            start_date: Some(Timestamp {
                seconds: 1629459200,
                nanos: 0,
            }),
            end_date: Some(Timestamp {
                seconds: 1632055200,
                nanos: 0,
            }),
            min_quantity: 10,
            max_quantity: Some(100),
            offer_prices: vec![offer_messages::OfferPrice {
                price: 10.5,
                currency: "USD".to_string(),
            }],
        };
        let model_offer = map_proto_offer_to_model_offer(offer_create_request);
        println!("model_offer: {:?}", model_offer);
    }

    #[test]
    fn test_map_model_offer_to_proto_offer() {
        let model_offer = model::Offer {
            id: Some(Uuid::new_v4().to_string()),
            item_id: "item1".to_string(),
            item_ref: Some("item_ref1".to_string()),
            start_date: Utc::now(),
            end_date: Utc::now(),
            min_quantity: 10,
            max_quantity: Some(100),
            offer_prices: vec![model::OfferPrice {
                price: Decimal128::from_str("10.5").unwrap(),
                currency: Currency::USD,
            }],
        };
        let proto_offer = map_model_offer_to_proto_offer(model_offer);
        println!("proto_offer: {:?}", proto_offer);
    }
}
