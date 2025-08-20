use std::{collections::HashMap, future::Future, pin::Pin, sync::Arc};

use async_nats::{Client, Message};
// use axum::{http::StatusCode, response::IntoResponse};
use log::{debug, error, warn};
use prost::Message as ProstMessage;
// use prost::Message;

use crate::{
    model::{self},
    order_messages::{self, OrderGetResponse},
    persistence::orders_dao::OrdersDaoImpl,
};

mod handlers_inner;

// impl IntoResponse for handlers_inner::HandlerError {
//     fn into_response(self) -> axum::response::Response {
//         match self {
//             handlers_inner::HandlerError::BadRequest(message) => {
//                 (StatusCode::BAD_REQUEST, message).into_response()
//             }
//             handlers_inner::HandlerError::InternalError(message) => {
//                 (StatusCode::INTERNAL_SERVER_ERROR, message).into_response()
//             }
//         }
//     }
// }

pub type NatsFn = Box<
    dyn Fn(Client, Arc<OrdersDaoImpl>, Message) -> Pin<Box<dyn Future<Output = ()> + Send>>
        + Send
        + Sync,
>;

pub struct Router {
    pub route_map: HashMap<String, NatsFn>,
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
    pub fn add_route(&mut self, path: String, f: NatsFn) -> &mut Self {
        self.route_map.insert(path, f);
        self
    }
    #[allow(dead_code)]
    pub async fn route(
        client: Client,
        routes: &HashMap<String, NatsFn>,
        path: String,
        orders_dao: Arc<OrdersDaoImpl>,
        request: Message,
    ) {
        let r = routes.get(&path).unwrap();
        r(client, orders_dao.clone(), request).await;
    }
}

// Create order
pub async fn create_order(
    client: Client,
    orders_dao: Arc<OrdersDaoImpl>,
    order_create_request: Message,
) {
    let order = order_messages::OrderCreateRequest::decode(order_create_request.payload.clone());
    match order {
        Ok(order) => {
            let mut model_addr: Option<model::Address> = None;
            if order.sold_to.is_some() {
                let req_addr = order.sold_to.unwrap();
                let mut addr_bldr = model::AddressBuilder::new(
                    req_addr.id,
                    req_addr.name,
                    req_addr.address_line1,
                    req_addr.city,
                    req_addr.postal_code,
                    req_addr.country,
                    req_addr.telephone,
                );
                if req_addr.address_line2.is_some() {
                    addr_bldr.address_line2(req_addr.address_line2.unwrap());
                }
                if req_addr.company.is_some() {
                    addr_bldr.company(req_addr.company.unwrap());
                }
                if req_addr.state_province.is_some() {
                    addr_bldr.state_province(req_addr.state_province.unwrap());
                }
                if req_addr.email.is_some() {
                    addr_bldr.email(req_addr.email.unwrap());
                }
                model_addr = Some(addr_bldr.build());
            }

            let ocr = model::OrderCreateRequest {
                order_ref: order.order_ref.clone(),
                sold_to: model_addr,
                order_items: None,
            };

            let result = handlers_inner::create_order(ocr, orders_dao.as_ref()).await;
            match result {
                Ok(o) => {
                    let ocrsp = order_messages::OrderCreateResponse {
                        order: Some(map_model_order_to_proto_order(o)),
                        status: Some(order_messages::Status {
                            code: order_messages::Code::Ok.into(),
                            message: "Order retrieved".to_string(),
                            details: vec![],
                        }),
                    };

                    let mut buf = vec![];
                    ocrsp.encode(&mut buf).unwrap();
                    client
                        .publish(order_create_request.reply.unwrap(), buf.into())
                        .await
                        .unwrap();
                    // order_create_request.respond(Ok(buf.into())).await.unwrap();
                }
                Err(err) => {
                    match err {
                        handlers_inner::HandlerError::InternalError(msg) => {
                            error!("Error creating order: {msg}");
                            client
                                .publish(
                                    order_create_request.reply.unwrap(),
                                    "Internal server error".into(),
                                )
                                .await
                                .unwrap();
                        }
                    };
                }
            }
        }
        Err(err) => {
            warn!("Invalid order format: {err:?}");
            client
                .publish(
                    order_create_request.reply.unwrap(),
                    "Message could not be decoded".into(),
                )
                .await
                .unwrap();
        }
    }
}

// Get order
pub async fn get_order(client: Client, orders_dao: Arc<OrdersDaoImpl>, order_get_request: Message) {
    let request = order_messages::OrderGetRequest::decode(order_get_request.payload.clone());
    match request {
        Ok(request) => {
            let result = handlers_inner::get_order(request.id.clone(), orders_dao.as_ref()).await;
            match result {
                Ok(Some(order)) => {
                    let order_get_response = OrderGetResponse {
                        order: Some(map_model_order_to_proto_order(order)),
                        status: Some(order_messages::Status {
                            code: order_messages::Code::Ok.into(),
                            message: "Order retrieved".to_string(),
                            details: vec![],
                        }),
                    };
                    let mut buf = vec![];
                    order_get_response.encode(&mut buf).unwrap();
                    client
                        .publish(order_get_request.reply.unwrap(), buf.into())
                        .await
                        .unwrap();
                }
                Ok(None) => {
                    let order_get_response = OrderGetResponse {
                        order: None,
                        status: Some(order_messages::Status {
                            code: order_messages::Code::NotFound.into(),
                            message: "Order not found".to_string(),
                            details: vec![],
                        }),
                    };
                    let mut buf = vec![];
                    order_get_response.encode(&mut buf).unwrap();
                    client
                        .publish(order_get_request.reply.unwrap(), buf.into())
                        .await
                        .unwrap();
                }
        Err(_) => {
            error!("Error getting order {}", request.id.clone());
                    let order_get_response = OrderGetResponse {
                        order: None,
                        status: Some(order_messages::Status {
                            code: order_messages::Code::Internal.into(),
                            message: "Failed to get order".to_owned(),
                            details: vec![],
                        }),
                    };
                    let mut buf = vec![];
                    order_get_response.encode(&mut buf).unwrap();
                    client
                        .publish(order_get_request.reply.unwrap(), buf.into())
                        .await
                        .unwrap();
                }
            }
        }
        Err(err) => {
        error!("Error decoding message: {err:?}");
            let order_get_response = OrderGetResponse {
                order: None,
                status: Some(order_messages::Status {
                    code: order_messages::Code::InvalidArgument.into(),
            message: format!("Invalid OrderGetRequest format {err:?}"),
                    details: vec![],
                }),
            };
            let mut buf = vec![];
            order_get_response.encode(&mut buf).unwrap();
            client
                .publish(order_get_request.reply.unwrap(), buf.into())
                .await
                .unwrap();
        }
    }
}

// Delete order
pub async fn delete_order(
    client: Client,
    orders_dao: Arc<OrdersDaoImpl>,
    order_delete_request: Message,
) {
    let request = order_messages::OrderDeleteRequest::decode(order_delete_request.payload.clone());
    debug!("start of delete order");
    match request {
        Ok(request) => {
            let order_id = request.id;
            debug!("order_id: {order_id}");
            let result = handlers_inner::delete_order(order_id, orders_dao.as_ref()).await;
            match result {
                Ok(_) => {
                    debug!("start of delete order Ok block");
                    let odresp = order_messages::OrderDeleteResponse {
                        status: Some(order_messages::Status {
                            code: order_messages::Code::Ok.into(),
                            message: "Order deleted".to_string(),
                            details: vec![],
                        }),
                    };
                    let mut buf = vec![];
                    odresp.encode(&mut buf).unwrap();
                    client
                        .publish(order_delete_request.reply.unwrap(), buf.into())
                        .await
                        .unwrap();
                }
        Err(e) => {
                    match e {
                        handlers_inner::HandlerError::InternalError(msg) => {
                error!("Internal error deleting order: {msg:?}");
                            let odresp = order_messages::OrderDeleteResponse {
                                status: Some(order_messages::Status {
                                    code: order_messages::Code::Internal.into(),
                    message: format!("Internal error deleting order {msg:?}"),
                                    details: vec![],
                                }),
                            };
                            let mut buf = vec![];
                            odresp.encode(&mut buf).unwrap();
                            client
                                .publish(order_delete_request.reply.unwrap(), buf.into())
                                .await
                                .unwrap();
                        }
                    };
                }
            }
        }
        Err(err) => {
        error!("Invalid OrderDeleteRequest format: {err:?}");
            let odresp = order_messages::OrderDeleteResponse {
                status: Some(order_messages::Status {
                    code: order_messages::Code::InvalidArgument.into(),
            message: format!("Invalid OrderDeleteRequest format {err:?}"),
                    details: vec![],
                }),
            };
            let mut buf = vec![];
            odresp.encode(&mut buf).unwrap();
            client
                .publish(order_delete_request.reply.unwrap(), buf.into())
                .await
                .unwrap();
        }
    }
}

// Translates a mode::Order to protobuf order - order_messages::Order
fn map_model_order_to_proto_order(order: model::Order) -> order_messages::Order {
    order_messages::Order {
        id: order.id,
        order_ref: order.order_ref,
        sold_to: order.sold_to.map(|a| order_messages::Address {
            id: a.id,
            customer_ref: a.customer_ref,
            name: a.name,
            address_line1: a.address_line1,
            address_line2: a.address_line2,
            company: a.company,
            city: a.city,
            state_province: a.state_province,
            postal_code: a.postal_code,
            country: a.country,
            telephone: a.telephone,
            email: a.email,
        }),
        bill_to: order.bill_to.map(|a| order_messages::Address {
            id: a.id,
            customer_ref: a.customer_ref,
            name: a.name,
            address_line1: a.address_line1,
            address_line2: a.address_line2,
            company: a.company,
            city: a.city,
            state_province: a.state_province,
            postal_code: a.postal_code,
            country: a.country,
            telephone: a.telephone,
            email: a.email,
        }),
        order_items: order.order_items.map_or(vec![], |vec_oi| {
            vec_oi
                .iter()
                .map(|oi| order_messages::OrderItem {
                    line_num: oi.line_num,
                    order_id: oi.order_id.clone(),
                    quantity: oi.quantity,
                    item: {
                        Some(order_messages::Item {
                            id: oi.item.id.clone(),
                            item_ref: oi.item.item_ref.clone(),
                            product_id: oi.item.product_id.clone(),
                            product_ref: oi.item.product_ref.clone(),
                            name: oi.item.name.clone(),
                            image_url: oi.item.image_url.clone(),
                            attributes: oi.item.attributes.clone().map_or(vec![], |vec_attr| {
                                vec_attr
                                    .iter()
                                    .map(|attr| order_messages::Attribute {
                                        seq: attr.seq,
                                        attribute_ref: attr.attribute_ref.clone(),
                                        name: attr.name.clone(),
                                        value: attr.value.clone(),
                                    })
                                    .collect()
                            }),
                            product_display_url: oi.item.product_display_url.clone(),
                        })
                    },
                    price: {
                        Some(order_messages::Price {
                            id: oi.price.id.clone(),
                            amount: oi.price.amount,
                            currency: oi.price.currency.clone(),
                        })
                    },
                    ship_to: oi.ship_to.clone().map(|a| order_messages::Address {
                        id: a.id,
                        customer_ref: a.customer_ref,
                        name: a.name,
                        address_line1: a.address_line1,
                        address_line2: a.address_line2,
                        company: a.company,
                        city: a.city,
                        state_province: a.state_province,
                        postal_code: a.postal_code,
                        country: a.country,
                        telephone: a.telephone,
                        email: a.email,
                    }),
                    orderitem_totals: oi.orderitem_totals.clone().map(|ot| {
                        order_messages::OrderTotals {
                            product_total: ot.product_total,
                            tax_total: ot.tax_total,
                            shipping_total: ot.shipping_total,
                            discount_total: ot.discount_total,
                        }
                    }),
                })
                .collect()
        }),
        order_totals: order.order_totals.map(|ot| order_messages::OrderTotals {
            product_total: ot.product_total,
            tax_total: ot.tax_total,
            shipping_total: ot.shipping_total,
            discount_total: ot.discount_total,
        }),
    }
}
