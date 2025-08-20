use std::future::Future;
use std::{collections::HashMap, sync::Arc};

use async_nats::{Client, Subject};
use bytes::Bytes;
use chrono::{Timelike, Utc};
use futures::future::BoxFuture;
use log::{debug, error};
use prost::Message as ProstMessage;
use prost_types::Timestamp;
use uuid::Uuid;

use crate::{
    inventory_messages::{self},
    model,
    persistence::inventory_dao::InventoryDaoImpl,
};

pub mod handlers_inner;

pub type Request = async_nats::Message;

#[derive(Clone)]
pub struct Response {
    pub subject: Subject,
    pub payload: Bytes,
}

pub trait HandlerFn: Send + Sync {
    fn call(&self, dao: Arc<InventoryDaoImpl>, req: Request) -> BoxFuture<'static, Response>;
}

impl<T, F> HandlerFn for T
where
    T: Fn(Arc<InventoryDaoImpl>, Request) -> F + Sync + Send + 'static,
    F: Future<Output = Response> + 'static + Send,
{
    fn call(&self, dao: Arc<InventoryDaoImpl>, req: Request) -> BoxFuture<'static, Response> {
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
}

impl Default for Router {
    fn default() -> Self {
        Self::new()
    }
}

impl Router {
    pub fn add_route(&mut self, path: String, f: Box<dyn HandlerFn>) -> &mut Self {
        self.route_map.insert(path, f);
        self
    }
    #[allow(dead_code)]
    pub async fn route(
        client: Client,
        routes: &RouteMap,
        path: String,
        inventory_dao: Arc<InventoryDaoImpl>,
        request: Request,
    ) {
        let r = routes.get(&path).unwrap();
        let response = r.call(inventory_dao, request).await;
        client
            .publish(response.subject, response.payload)
            .await
            .unwrap();
    }
}

pub async fn create_item(
    inventory_dao: Arc<InventoryDaoImpl>,
    inventory_create_request: Request,
) -> Response {
    let item = inventory_messages::InventoryCreateRequest::decode(
        inventory_create_request.payload.clone(),
    );
    let mut inventory_create_response = inventory_messages::InventoryCreateResponse {
        ..Default::default()
    };
    match item {
        Ok(item) => {
            debug!("inventory item: {item:?}");
            let model_item = map_proto_item_to_model_item(item);

            let result = handlers_inner::create_item(model_item, inventory_dao.as_ref()).await;
            match result {
                Ok(i) => {
                    inventory_create_response.item = Some(map_model_item_to_proto_item(i));
                    inventory_create_response.status = Some(inventory_messages::Status {
                        code: inventory_messages::Code::Ok.into(),
                        message: "".to_owned(),
                        details: vec![],
                    });
                }
                Err(err) => match err {
                    handlers_inner::HandlerError::InternalError(msg) => {
                        inventory_create_response.status = Some(inventory_messages::Status {
                            code: inventory_messages::Code::Internal.into(),
                            message: msg,
                            details: vec![],
                        });
                    }
                },
            }
        }
        Err(e) => {
            error!("Error decoding inventory create request: {e}");
            inventory_create_response.status = Some(inventory_messages::Status {
                code: inventory_messages::Code::InvalidArgument.into(),
                message: "Failed to decode request".to_owned(),
                details: vec![],
            });
        }
    }

    Response {
        subject: inventory_create_request.reply.unwrap(),
        payload: Bytes::from(inventory_create_response.encode_to_vec()),
    }
}

pub async fn get_item(
    inventory_dao: Arc<InventoryDaoImpl>,
    inventory_get_request: Request,
) -> Response {
    let item =
        inventory_messages::InventoryGetRequest::decode(inventory_get_request.payload.clone());
    let mut inventory_get_response = inventory_messages::InventoryGetResponse {
        ..Default::default()
    };

    match item {
        Ok(item) => {
            debug!("get inventory item: {item:?}");
            let result = handlers_inner::get_item(item.sku, inventory_dao.as_ref()).await;
            match result {
                Ok(Some(i)) => {
                    inventory_get_response.item = Some(map_model_item_to_proto_item(i));
                    inventory_get_response.status = Some(inventory_messages::Status {
                        code: inventory_messages::Code::Ok.into(),
                        message: "".to_owned(),
                        details: vec![],
                    });
                }
                Ok(None) => {
                    inventory_get_response.status = Some(inventory_messages::Status {
                        code: inventory_messages::Code::NotFound.into(),
                        message: "Inventory item not found".to_owned(),
                        details: vec![],
                    });
                }
                Err(err) => match err {
                    handlers_inner::HandlerError::InternalError(msg) => {
                        inventory_get_response.status = Some(inventory_messages::Status {
                            code: inventory_messages::Code::Internal.into(),
                            message: msg,
                            details: vec![],
                        });
                    }
                },
            }
        }
        Err(e) => {
            error!("Error decoding inventory get request: {e}");
            inventory_get_response.status = Some(inventory_messages::Status {
                code: inventory_messages::Code::InvalidArgument.into(),
                message: "Failed to decode request".to_owned(),
                details: vec![],
            });
        }
    }

    Response {
        subject: inventory_get_request.reply.unwrap(),
        payload: Bytes::from(inventory_get_response.encode_to_vec()),
    }
}

pub async fn delete_item(
    inventory_dao: Arc<InventoryDaoImpl>,
    inventory_delete_request: Request,
) -> Response {
    let item = inventory_messages::InventoryDeleteRequest::decode(
        inventory_delete_request.payload.clone(),
    );
    let mut inventory_delete_response = inventory_messages::InventoryDeleteResponse {
        ..Default::default()
    };

    match item {
        Ok(item) => {
            debug!("delete inventory item: {item:?}");
            let result = handlers_inner::delete_item(item.sku, inventory_dao.as_ref()).await;
            match result {
                Ok(_) => {
                    inventory_delete_response.status = Some(inventory_messages::Status {
                        code: inventory_messages::Code::Ok.into(),
                        message: "".to_owned(),
                        details: vec![],
                    });
                }
                Err(err) => match err {
                    handlers_inner::HandlerError::InternalError(msg) => {
                        inventory_delete_response.status = Some(inventory_messages::Status {
                            code: inventory_messages::Code::Internal.into(),
                            message: msg,
                            details: vec![],
                        });
                    }
                },
            }
        }
        Err(e) => {
            error!("Error decoding inventory delete request: {e}");
            inventory_delete_response.status = Some(inventory_messages::Status {
                code: inventory_messages::Code::InvalidArgument.into(),
                message: "Failed to decode request".to_owned(),
                details: vec![],
            });
        }
    }

    Response {
        subject: inventory_delete_request.reply.unwrap(),
        payload: Bytes::from(inventory_delete_response.encode_to_vec()),
    }
}

pub async fn update_stock(
    inventory_dao: Arc<InventoryDaoImpl>,
    inventory_update_request: Request,
) -> Response {
    let update = inventory_messages::InventoryUpdateStockRequest::decode(
        inventory_update_request.payload.clone(),
    );
    let mut inventory_update_response = inventory_messages::InventoryUpdateStockResponse {
        ..Default::default()
    };

    match update {
        Ok(update) => {
            debug!("update inventory stock: {update:?}");
            let result = handlers_inner::update_stock(
                update.sku,
                update.quantity_change,
                update.reason,
                inventory_dao.as_ref(),
            )
            .await;
            match result {
                Ok(Some(i)) => {
                    inventory_update_response.item = Some(map_model_item_to_proto_item(i));
                    inventory_update_response.status = Some(inventory_messages::Status {
                        code: inventory_messages::Code::Ok.into(),
                        message: "".to_owned(),
                        details: vec![],
                    });
                }
                Ok(None) => {
                    inventory_update_response.status = Some(inventory_messages::Status {
                        code: inventory_messages::Code::NotFound.into(),
                        message: "Inventory item not found".to_owned(),
                        details: vec![],
                    });
                }
                Err(err) => match err {
                    handlers_inner::HandlerError::InternalError(msg) => {
                        inventory_update_response.status = Some(inventory_messages::Status {
                            code: inventory_messages::Code::Internal.into(),
                            message: msg,
                            details: vec![],
                        });
                    }
                },
            }
        }
        Err(e) => {
            error!("Error decoding inventory update request: {e}");
            inventory_update_response.status = Some(inventory_messages::Status {
                code: inventory_messages::Code::InvalidArgument.into(),
                message: "Failed to decode request".to_owned(),
                details: vec![],
            });
        }
    }

    Response {
        subject: inventory_update_request.reply.unwrap(),
        payload: Bytes::from(inventory_update_response.encode_to_vec()),
    }
}

pub async fn get_all_locations_by_sku(
    inventory_dao: Arc<InventoryDaoImpl>,
    request: Request,
) -> Response {
    let request_data =
        inventory_messages::InventoryGetAllLocationsBySkuRequest::decode(request.payload.clone());
    let mut response = inventory_messages::InventoryGetAllLocationsBySkuResponse {
        ..Default::default()
    };

    match request_data {
        Ok(req) => {
            debug!("get all locations by sku request: {req:?}");

            // Validate SKU count (max 100)
            if req.skus.len() > 100 {
                response.status = Some(inventory_messages::Status {
                    code: inventory_messages::Code::InvalidArgument.into(),
                    message: "Maximum 100 SKUs allowed per request".to_owned(),
                    details: vec![],
                });
                return Response {
                    subject: request.reply.unwrap(),
                    payload: Bytes::from(response.encode_to_vec()),
                };
            }

            // Validate that SKUs array is not empty
            if req.skus.is_empty() {
                response.status = Some(inventory_messages::Status {
                    code: inventory_messages::Code::InvalidArgument.into(),
                    message: "SKUs array cannot be empty".to_owned(),
                    details: vec![],
                });
                return Response {
                    subject: request.reply.unwrap(),
                    payload: Bytes::from(response.encode_to_vec()),
                };
            }

            let result =
                handlers_inner::get_all_locations_by_sku(req.skus.clone(), inventory_dao.as_ref())
                    .await;
            match result {
                Ok((inventory_by_sku, not_found_skus)) => {
                    let mut sku_summaries = Vec::new();

                    // Process each SKU that was found
                    for (sku, items) in inventory_by_sku {
                        if !items.is_empty() {
                            // Calculate aggregated totals
                            let total_quantity: i32 = items.iter().map(|i| i.quantity).sum();
                            let total_reserved: i32 =
                                items.iter().map(|i| i.reserved_quantity).sum();
                            let total_available: i32 =
                                items.iter().map(|i| i.available_quantity).sum();
                            let min_stock_level: i32 =
                                items.iter().map(|i| i.min_stock_level).min().unwrap_or(0);
                            let location_count = items.len() as i32;

                            // Create location details
                            let location_details: Vec<inventory_messages::InventoryLocationDetail> =
                                items
                                    .into_iter()
                                    .map(|item| inventory_messages::InventoryLocationDetail {
                                        location: item.location,
                                        quantity: item.quantity,
                                        reserved_quantity: item.reserved_quantity,
                                        available_quantity: item.available_quantity,
                                        min_stock_level: item.min_stock_level,
                                        last_updated: Some(Timestamp {
                                            seconds: item.last_updated.timestamp(),
                                            nanos: item.last_updated.nanosecond() as i32,
                                        }),
                                        created_at: Some(Timestamp {
                                            seconds: item.created_at.timestamp(),
                                            nanos: item.created_at.nanosecond() as i32,
                                        }),
                                    })
                                    .collect();

                            // Create SKU summary
                            let sku_summary = inventory_messages::SkuInventorySummary {
                                sku: sku.clone(),
                                total_inventory: Some(inventory_messages::InventoryAggregation {
                                    total_quantity,
                                    total_reserved_quantity: total_reserved,
                                    total_available_quantity: total_available,
                                    min_stock_level_across_locations: min_stock_level,
                                    location_count,
                                }),
                                location_details,
                            };

                            sku_summaries.push(sku_summary);
                        }
                    }

                    response.sku_summaries = sku_summaries;
                    response.not_found_skus = not_found_skus;

                    response.status = Some(inventory_messages::Status {
                        code: inventory_messages::Code::Ok.into(),
                        message: "".to_owned(),
                        details: vec![],
                    });
                }
                Err(err) => match err {
                    handlers_inner::HandlerError::InternalError(msg) => {
                        response.status = Some(inventory_messages::Status {
                            code: inventory_messages::Code::Internal.into(),
                            message: msg,
                            details: vec![],
                        });
                    }
                },
            }
        }
        Err(e) => {
            error!("Error decoding get all locations by sku request: {e}");
            response.status = Some(inventory_messages::Status {
                code: inventory_messages::Code::InvalidArgument.into(),
                message: "Failed to decode request".to_owned(),
                details: vec![],
            });
        }
    }

    Response {
        subject: request.reply.unwrap(),
        payload: Bytes::from(response.encode_to_vec()),
    }
}

// Helper functions to map between protobuf and model types
fn map_proto_item_to_model_item(
    proto_item: inventory_messages::InventoryCreateRequest,
) -> model::InventoryItem {
    let now = Utc::now();
    model::InventoryItem {
        id: Some(Uuid::new_v4().to_string()),
        sku: proto_item.sku,
        quantity: proto_item.quantity,
        reserved_quantity: proto_item.reserved_quantity,
        available_quantity: proto_item.quantity - proto_item.reserved_quantity,
        min_stock_level: proto_item.min_stock_level,
        location: proto_item.location,
        last_updated: now,
        created_at: now,
    }
}

fn map_model_item_to_proto_item(
    model_item: model::InventoryItem,
) -> inventory_messages::InventoryItem {
    inventory_messages::InventoryItem {
        id: model_item.id,
        sku: model_item.sku,
        quantity: model_item.quantity,
        reserved_quantity: model_item.reserved_quantity,
        available_quantity: model_item.available_quantity,
        min_stock_level: model_item.min_stock_level,
        location: model_item.location,
        last_updated: Some(Timestamp {
            seconds: model_item.last_updated.timestamp(),
            nanos: model_item.last_updated.nanosecond() as i32,
        }),
        created_at: Some(Timestamp {
            seconds: model_item.created_at.timestamp(),
            nanos: model_item.created_at.nanosecond() as i32,
        }),
    }
}
