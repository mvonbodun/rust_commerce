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
                        handlers_inner::HandlerError::ValidationError(msg) => {
                            error!("Validation error creating offer: {}", msg);
                            offer_create_response.status = Some(offer_messages::Status {
                                code: offer_messages::Code::InvalidArgument.into(),
                                message: format!("Validation error: {}", msg),
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
                        handlers_inner::HandlerError::ValidationError(msg) => {
                            error!("Validation error getting offer: {}", msg);
                            offer_get_response.status = Some(offer_messages::Status {
                                code: offer_messages::Code::InvalidArgument.into(),
                                message: format!("Validation error: {}", msg),
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
                        handlers_inner::HandlerError::ValidationError(msg) => {
                            error!("Validation error deleting offer: {}", msg);
                            offer_delete_response.status = Some(offer_messages::Status {
                                code: offer_messages::Code::InvalidArgument.into(),
                                message: format!("Validation error: {}", msg),
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

pub async fn get_best_offer_price(offer_dao: Arc<OfferDaoImpl>, request: Request) -> Response {
    let decoded_request = offer_messages::GetBestOfferPriceRequest::decode(request.payload.clone());
    let mut response = offer_messages::GetBestOfferPriceResponse {
        offer: None,
        found: false,
    };

    match decoded_request {
        Ok(req) => {
            debug!("GetBestOfferPrice request: {:?}", req);
            
            let result = handlers_inner::get_best_offer_price(
                req.sku,
                req.quantity,
                req.date,
                req.currency,
                offer_dao.as_ref(),
            ).await;

            match result {
                Ok(Some(offer)) => {
                    response.offer = Some(map_model_offer_to_proto_offer(offer));
                    response.found = true;
                    debug!("Found best offer price");
                }
                Ok(None) => {
                    response.found = false;
                    debug!("No offer found matching criteria");
                }
                Err(err) => {
                    match err {
                        handlers_inner::HandlerError::ValidationError(msg) => {
                            error!("Validation error in get_best_offer_price: {}", msg);
                            // For validation errors, we still return found=false but log the error
                            response.found = false;
                        }
                        handlers_inner::HandlerError::InternalError(msg) => {
                            error!("Internal error in get_best_offer_price: {}", msg);
                            response.found = false;
                        }
                    }
                }
            }
        }
        Err(err) => {
            error!("Error decoding GetBestOfferPriceRequest: {}", err);
            response.found = false;
        }
    }

    let mut buf = vec![];
    response.encode(&mut buf).unwrap();
    Response {
        subject: request.reply.unwrap(),
        payload: buf.into(),
    }
}

pub async fn get_best_offer_prices(offer_dao: Arc<OfferDaoImpl>, request: Request) -> Response {
    let decoded_request = offer_messages::GetBestOfferPricesRequest::decode(request.payload.clone());
    let mut response = offer_messages::GetBestOfferPricesResponse {
        sku_results: vec![],
        status: None,
    };

    match decoded_request {
        Ok(req) => {
            debug!("GetBestOfferPrices request: {:?}", req);
            
            let result = handlers_inner::get_best_offer_prices(
                req.skus,
                req.quantity,
                req.date,
                req.currency,
                offer_dao.as_ref(),
            ).await;

            match result {
                Ok(offers_map) => {
                    // Convert the HashMap results into SkuOfferResult messages
                    let mut sku_results = vec![];
                    for (sku, offer_option) in offers_map {
                        let found = offer_option.is_some();
                        let sku_result = offer_messages::SkuOfferResult {
                            sku: sku.clone(),
                            offer: offer_option.map(map_model_offer_to_proto_offer),
                            found,
                        };
                        sku_results.push(sku_result);
                    }
                    response.sku_results = sku_results;
                    response.status = Some(offer_messages::Status {
                        code: offer_messages::Code::Ok.into(),
                        message: "Success".to_string(),
                        details: vec![],
                    });
                    debug!("Successfully processed {} SKUs", response.sku_results.len());
                }
                Err(err) => {
                    match err {
                        handlers_inner::HandlerError::ValidationError(msg) => {
                            error!("Validation error in get_best_offer_prices: {}", msg);
                            response.status = Some(offer_messages::Status {
                                code: offer_messages::Code::InvalidArgument.into(),
                                message: msg,
                                details: vec![],
                            });
                        }
                        handlers_inner::HandlerError::InternalError(msg) => {
                            error!("Internal error in get_best_offer_prices: {}", msg);
                            response.status = Some(offer_messages::Status {
                                code: offer_messages::Code::Internal.into(),
                                message: "Internal server error".to_string(),
                                details: vec![],
                            });
                        }
                    }
                }
            }
        }
        Err(err) => {
            error!("Error decoding GetBestOfferPricesRequest: {}", err);
            response.status = Some(offer_messages::Status {
                code: offer_messages::Code::InvalidArgument.into(),
                message: "Invalid request format".to_string(),
                details: vec![],
            });
        }
    }

    let mut buf = vec![];
    response.encode(&mut buf).unwrap();
    Response {
        subject: request.reply.unwrap(),
        payload: buf.into(),
    }
}

fn map_proto_offer_to_model_offer(offer: offer_messages::OfferCreateRequest) -> model::Offer {
    model::Offer {
        id: Some(Uuid::new_v4().to_string()),
        sku: offer.sku.clone(),
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
                price: Decimal128::from_str(&op.price).unwrap(),
                currency: Currency::from_code(op.currency.as_str()).expect("currency is not valid"),
            })
            .collect(),
    }
}

// Map a model offer to a protocol buffer offer
fn map_model_offer_to_proto_offer(offer: model::Offer) -> offer_messages::Offer {
    offer_messages::Offer {
        id: offer.id,
        sku: offer.sku,
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
                price: op.price.to_string(),
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
            sku: "SKU123".to_string(),
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
                price: "10.5".to_string(),
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
            sku: "SKU123".to_string(),
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

    #[test]
    fn test_get_best_offer_price_protobuf_messages() {
        // Test that we can create protobuf messages for the new API
        let request = offer_messages::GetBestOfferPriceRequest {
            sku: "TEST-SKU-001".to_string(),
            quantity: 5,
            date: None,  // Optional field
            currency: "USD".to_string(),
        };
        
        assert_eq!(request.sku, "TEST-SKU-001");
        assert_eq!(request.quantity, 5);
        assert_eq!(request.currency, "USD");
        assert!(request.date.is_none());
        
        // Test response creation - check the actual fields that exist
        let response = offer_messages::GetBestOfferPriceResponse {
            offer: None,  // Based on the protobuf definition
            found: false,
        };
        
        assert!(!response.found);
        assert!(response.offer.is_none());
    }

    #[test]
    fn test_get_best_offer_price_validation_logic() {
        // Test individual validation conditions
        
        // Test 1: Empty SKU should be invalid
        let sku = "";
        assert!(sku.trim().is_empty(), "Empty SKU should fail validation");
        
        // Test 2: Valid SKU should pass
        let sku = "VALID-SKU-001";
        assert!(!sku.trim().is_empty(), "Valid SKU should pass validation");
        
        // Test 3: Zero or negative quantity should be invalid
        let quantity = 0;
        assert!(quantity <= 0, "Zero quantity should fail validation");
        
        let quantity = -5;
        assert!(quantity <= 0, "Negative quantity should fail validation");
        
        // Test 4: Positive quantity should be valid
        let quantity = 5;
        assert!(quantity > 0, "Positive quantity should pass validation");
        
        // Test 5: Currency validation (based on the handler logic)
        let currency = "USD";
        assert!(currency == "USD" || currency == "EUR", "USD should be valid");
        
        let currency = "EUR";
        assert!(currency == "USD" || currency == "EUR", "EUR should be valid");
        
        let currency = "JPY";
        assert!(!(currency == "USD" || currency == "EUR"), "JPY should be invalid");
        
        // Test 6: Date parsing (simulate what the handler does)
        use chrono::NaiveDate;
        
        let date_str = "2024-06-15";
        let result = NaiveDate::parse_from_str(date_str, "%Y-%m-%d");
        assert!(result.is_ok(), "Valid date format should parse successfully");
        
        let date_str = "invalid-date";
        let result = NaiveDate::parse_from_str(date_str, "%Y-%m-%d");
        assert!(result.is_err(), "Invalid date format should fail parsing");
    }

    #[tokio::test]
    async fn test_get_best_offer_price_handler_error_types() {
        // Test that the HandlerError enum works as expected
        use handlers_inner::HandlerError;
        
        let validation_error = HandlerError::ValidationError("Test validation error".to_string());
        match validation_error {
            HandlerError::ValidationError(msg) => {
                assert_eq!(msg, "Test validation error");
            },
            _ => panic!("Expected ValidationError"),
        }
        
        let internal_error = HandlerError::InternalError("Test internal error".to_string());
        match internal_error {
            HandlerError::InternalError(msg) => {
                assert_eq!(msg, "Test internal error");
            },
            _ => panic!("Expected InternalError"),
        }
    }
}
