use std::{collections::HashMap, future::Future, pin::Pin, sync::Arc};

use async_nats::{Client, Message};
use log::{debug, error, warn};
use prost::Message as ProstMessage;

use crate::{
    catalog_messages::{
        self, GetProductSlugsRequest, GetProductSlugsResponse, ProductCreateRequest,
        ProductCreateResponse, ProductDeleteRequest, ProductDeleteResponse, ProductExportRequest,
        ProductExportResponse, ProductGetBySlugRequest, ProductGetBySlugResponse,
        ProductGetRequest, ProductGetResponse, ProductSearchRequest, ProductSearchResponse,
        ProductUpdateRequest, ProductUpdateResponse,
    },
    model::Product,
    persistence::product_dao::ProductDao,
};

pub mod category_handlers;
pub mod category_service;
mod handlers_inner;

pub type NatsFn = Box<
    dyn Fn(
            Arc<dyn ProductDao + Send + Sync>,
            Client,
            Message,
        ) -> Pin<
            Box<dyn Future<Output = Result<(), Box<dyn std::error::Error + Send + Sync>>> + Send>,
        > + Send
        + Sync,
>;

pub struct Router {
    pub route_map: HashMap<String, NatsFn>,
}

impl Router {
    pub fn new() -> Self {
        Self {
            route_map: HashMap::new(),
        }
    }

    pub fn add_route(&mut self, path: String, handler: NatsFn) -> &mut Self {
        self.route_map.insert(path, handler);
        self
    }
}

pub async fn create_product(
    product_dao: Arc<dyn ProductDao + Send + Sync>,
    client: Client,
    msg: Message,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    debug!("Processing create_product request");

    let request = ProductCreateRequest::decode(&*msg.payload);
    match request {
        Ok(request) => {
            let result = handlers_inner::create_product(
                request,
                None, // created_by - could be extracted from metadata
                product_dao.as_ref(),
            )
            .await;

            match result {
                Ok(product) => {
                    let response = ProductCreateResponse {
                        product: Some(map_model_product_to_proto_product(product)),
                        status: Some(catalog_messages::Status {
                            code: catalog_messages::Code::Ok.into(),
                            message: "Product created successfully".to_string(),
                            details: vec![],
                        }),
                    };

                    let response_bytes = response.encode_to_vec();

                    if let Some(reply) = msg.reply {
                        if let Err(e) = client.publish(reply, response_bytes.into()).await {
                            error!("Failed to send response: {e}");
                        }
                    }
                }
                Err(handlers_inner::HandlerError::ValidationError(error_msg)) => {
                    warn!("Validation error creating product: {error_msg}");
                    let response = ProductCreateResponse {
                        product: None,
                        status: Some(catalog_messages::Status {
                            code: catalog_messages::Code::InvalidArgument.into(),
                            message: error_msg,
                            details: vec![],
                        }),
                    };
                    let response_bytes = response.encode_to_vec();

                    if let Some(reply) = msg.reply {
                        if let Err(e) = client.publish(reply, response_bytes.into()).await {
                            error!("Failed to send error response: {e}");
                        }
                    }
                }
                Err(handlers_inner::HandlerError::InternalError(error_msg)) => {
                    error!("Error creating product: {error_msg}");
                    let response = ProductCreateResponse {
                        product: None,
                        status: Some(catalog_messages::Status {
                            code: catalog_messages::Code::Internal.into(),
                            message: "Internal server error".to_string(),
                            details: vec![],
                        }),
                    };
                    let response_bytes = response.encode_to_vec();

                    if let Some(reply) = msg.reply {
                        if let Err(e) = client.publish(reply, response_bytes.into()).await {
                            error!("Failed to send error response: {e}");
                        }
                    }
                }
            }
        }
        Err(err) => {
            warn!("Invalid product create request format: {err:?}");
            let response = ProductCreateResponse {
                product: None,
                status: Some(catalog_messages::Status {
                    code: catalog_messages::Code::InvalidArgument.into(),
                    message: "Invalid request format".to_string(),
                    details: vec![],
                }),
            };
            let response_bytes = response.encode_to_vec();

            if let Some(reply) = msg.reply {
                if let Err(e) = client.publish(reply, response_bytes.into()).await {
                    error!("Failed to send error response: {e}");
                }
            }
        }
    }

    Ok(())
}

pub async fn get_product(
    product_dao: Arc<dyn ProductDao + Send + Sync>,
    client: Client,
    msg: Message,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    debug!("Processing get_product request");

    let request = ProductGetRequest::decode(&*msg.payload);
    match request {
        Ok(request) => {
            let result = handlers_inner::get_product(request.id, product_dao.as_ref()).await;

            match result {
                Ok(Some(product)) => {
                    let response = ProductGetResponse {
                        product: Some(map_model_product_to_proto_product(product)),
                        status: Some(catalog_messages::Status {
                            code: catalog_messages::Code::Ok.into(),
                            message: "Product retrieved successfully".to_string(),
                            details: vec![],
                        }),
                    };

                    let response_bytes = response.encode_to_vec();

                    if let Some(reply) = msg.reply {
                        if let Err(e) = client.publish(reply, response_bytes.into()).await {
                            error!("Failed to send response: {e}");
                        }
                    }
                }
                Ok(None) => {
                    let response = ProductGetResponse {
                        product: None,
                        status: Some(catalog_messages::Status {
                            code: catalog_messages::Code::NotFound.into(),
                            message: "Product not found".to_string(),
                            details: vec![],
                        }),
                    };
                    let response_bytes = response.encode_to_vec();

                    if let Some(reply) = msg.reply {
                        if let Err(e) = client.publish(reply, response_bytes.into()).await {
                            error!("Failed to send response: {e}");
                        }
                    }
                }
                Err(handlers_inner::HandlerError::ValidationError(error_msg)) => {
                    warn!("Validation error getting product: {error_msg}");
                    let response = ProductGetResponse {
                        product: None,
                        status: Some(catalog_messages::Status {
                            code: catalog_messages::Code::InvalidArgument.into(),
                            message: error_msg,
                            details: vec![],
                        }),
                    };
                    let response_bytes = response.encode_to_vec();
                    if let Some(reply) = msg.reply {
                        if let Err(e) = client.publish(reply, response_bytes.into()).await {
                            error!("Failed to send error response: {e}");
                        }
                    }
                }
                Err(handlers_inner::HandlerError::InternalError(error_msg)) => {
                    error!("Error getting product: {error_msg}");
                    let response = ProductGetResponse {
                        product: None,
                        status: Some(catalog_messages::Status {
                            code: catalog_messages::Code::Internal.into(),
                            message: "Internal server error".to_string(),
                            details: vec![],
                        }),
                    };
                    let response_bytes = response.encode_to_vec();

                    if let Some(reply) = msg.reply {
                        if let Err(e) = client.publish(reply, response_bytes.into()).await {
                            error!("Failed to send error response: {e}");
                        }
                    }
                }
            }
        }
        Err(err) => {
            warn!("Invalid product get request format: {err:?}");
            let response = ProductGetResponse {
                product: None,
                status: Some(catalog_messages::Status {
                    code: catalog_messages::Code::InvalidArgument.into(),
                    message: "Invalid request format".to_string(),
                    details: vec![],
                }),
            };
            let response_bytes = response.encode_to_vec();

            if let Some(reply) = msg.reply {
                if let Err(e) = client.publish(reply, response_bytes.into()).await {
                    error!("Failed to send error response: {e}");
                }
            }
        }
    }

    Ok(())
}

pub async fn get_product_by_slug(
    product_dao: Arc<dyn ProductDao + Send + Sync>,
    client: Client,
    msg: Message,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    debug!("Processing get_product_by_slug request");

    let request = ProductGetBySlugRequest::decode(&*msg.payload);
    match request {
        Ok(request) => {
            let result =
                handlers_inner::get_product_by_slug(request.slug, product_dao.as_ref()).await;

            match result {
                Ok(Some(product)) => {
                    let response = ProductGetBySlugResponse {
                        product: Some(map_model_product_to_proto_product(product)),
                        status: Some(catalog_messages::Status {
                            code: catalog_messages::Code::Ok.into(),
                            message: "Product retrieved successfully".to_string(),
                            details: vec![],
                        }),
                    };

                    let response_bytes = response.encode_to_vec();

                    if let Some(reply) = msg.reply {
                        if let Err(e) = client.publish(reply, response_bytes.into()).await {
                            error!("Failed to send response: {e}");
                        }
                    }
                }
                Ok(None) => {
                    let response = ProductGetBySlugResponse {
                        product: None,
                        status: Some(catalog_messages::Status {
                            code: catalog_messages::Code::NotFound.into(),
                            message: "Product not found".to_string(),
                            details: vec![],
                        }),
                    };

                    let response_bytes = response.encode_to_vec();

                    if let Some(reply) = msg.reply {
                        if let Err(e) = client.publish(reply, response_bytes.into()).await {
                            error!("Failed to send response: {e}");
                        }
                    }
                }
                Err(handlers_inner::HandlerError::ValidationError(error_msg)) => {
                    warn!("Validation error getting product by slug: {error_msg}");
                    let response = ProductGetBySlugResponse {
                        product: None,
                        status: Some(catalog_messages::Status {
                            code: catalog_messages::Code::InvalidArgument.into(),
                            message: error_msg,
                            details: vec![],
                        }),
                    };
                    let response_bytes = response.encode_to_vec();
                    if let Some(reply) = msg.reply {
                        if let Err(e) = client.publish(reply, response_bytes.into()).await {
                            error!("Failed to send error response: {e}");
                        }
                    }
                }
                Err(handlers_inner::HandlerError::InternalError(error_msg)) => {
                    error!("Error getting product by slug: {error_msg}");
                    let response = ProductGetBySlugResponse {
                        product: None,
                        status: Some(catalog_messages::Status {
                            code: catalog_messages::Code::Internal.into(),
                            message: "Internal server error".to_string(),
                            details: vec![],
                        }),
                    };

                    let response_bytes = response.encode_to_vec();

                    if let Some(reply) = msg.reply {
                        if let Err(e) = client.publish(reply, response_bytes.into()).await {
                            error!("Failed to send error response: {e}");
                        }
                    }
                }
            }
        }
        Err(err) => {
            warn!("Invalid product get by slug request format: {err:?}");
            let response = ProductGetBySlugResponse {
                product: None,
                status: Some(catalog_messages::Status {
                    code: catalog_messages::Code::InvalidArgument.into(),
                    message: "Invalid request format".to_string(),
                    details: vec![],
                }),
            };

            let response_bytes = response.encode_to_vec();

            if let Some(reply) = msg.reply {
                if let Err(e) = client.publish(reply, response_bytes.into()).await {
                    error!("Failed to send error response: {e}");
                }
            }
        }
    }

    Ok(())
}

pub async fn update_product(
    product_dao: Arc<dyn ProductDao + Send + Sync>,
    client: Client,
    msg: Message,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    debug!("Processing update_product request");

    let request = ProductUpdateRequest::decode(&*msg.payload);
    match request {
        Ok(request) => {
            let result =
                handlers_inner::update_product(request.id.clone(), request, product_dao.as_ref())
                    .await;

            match result {
                Ok(Some(product)) => {
                    let response = ProductUpdateResponse {
                        product: Some(map_model_product_to_proto_product(product)),
                        status: Some(catalog_messages::Status {
                            code: catalog_messages::Code::Ok.into(),
                            message: "Product updated successfully".to_string(),
                            details: vec![],
                        }),
                    };

                    let response_bytes = response.encode_to_vec();

                    if let Some(reply) = msg.reply {
                        if let Err(e) = client.publish(reply, response_bytes.into()).await {
                            error!("Failed to send response: {e}");
                        }
                    }
                }
                Ok(None) => {
                    let response = ProductUpdateResponse {
                        product: None,
                        status: Some(catalog_messages::Status {
                            code: catalog_messages::Code::NotFound.into(),
                            message: "Product not found".to_string(),
                            details: vec![],
                        }),
                    };

                    let response_bytes = response.encode_to_vec();

                    if let Some(reply) = msg.reply {
                        if let Err(e) = client.publish(reply, response_bytes.into()).await {
                            error!("Failed to send response: {e}");
                        }
                    }
                }
                Err(handlers_inner::HandlerError::ValidationError(error_msg)) => {
                    warn!("Validation error updating product: {error_msg}");
                    let response = ProductUpdateResponse {
                        product: None,
                        status: Some(catalog_messages::Status {
                            code: catalog_messages::Code::InvalidArgument.into(),
                            message: error_msg,
                            details: vec![],
                        }),
                    };
                    let response_bytes = response.encode_to_vec();
                    if let Some(reply) = msg.reply {
                        if let Err(e) = client.publish(reply, response_bytes.into()).await {
                            error!("Failed to send error response: {e}");
                        }
                    }
                }
                Err(handlers_inner::HandlerError::InternalError(error_msg)) => {
                    error!("Error updating product: {error_msg}");
                    let response = ProductUpdateResponse {
                        product: None,
                        status: Some(catalog_messages::Status {
                            code: catalog_messages::Code::Internal.into(),
                            message: "Internal server error".to_string(),
                            details: vec![],
                        }),
                    };

                    let response_bytes = response.encode_to_vec();

                    if let Some(reply) = msg.reply {
                        if let Err(e) = client.publish(reply, response_bytes.into()).await {
                            error!("Failed to send error response: {e}");
                        }
                    }
                }
            }
        }
        Err(err) => {
            warn!("Invalid product update request format: {err:?}");
            let response = ProductUpdateResponse {
                product: None,
                status: Some(catalog_messages::Status {
                    code: catalog_messages::Code::InvalidArgument.into(),
                    message: "Invalid request format".to_string(),
                    details: vec![],
                }),
            };

            let response_bytes = response.encode_to_vec();

            if let Some(reply) = msg.reply {
                if let Err(e) = client.publish(reply, response_bytes.into()).await {
                    error!("Failed to send error response: {e}");
                }
            }
        }
    }

    Ok(())
}

pub async fn delete_product(
    product_dao: Arc<dyn ProductDao + Send + Sync>,
    client: Client,
    msg: Message,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    debug!("Processing delete_product request");

    let request = ProductDeleteRequest::decode(&*msg.payload);
    match request {
        Ok(request) => {
            let result = handlers_inner::delete_product(request.id, product_dao.as_ref()).await;

            match result {
                Ok(true) => {
                    let response = ProductDeleteResponse {
                        status: Some(catalog_messages::Status {
                            code: catalog_messages::Code::Ok.into(),
                            message: "Product deleted successfully".to_string(),
                            details: vec![],
                        }),
                    };

                    let response_bytes = response.encode_to_vec();

                    if let Some(reply) = msg.reply {
                        if let Err(e) = client.publish(reply, response_bytes.into()).await {
                            error!("Failed to send response: {e}");
                        }
                    }
                }
                Ok(false) => {
                    let response = ProductDeleteResponse {
                        status: Some(catalog_messages::Status {
                            code: catalog_messages::Code::NotFound.into(),
                            message: "Product not found".to_string(),
                            details: vec![],
                        }),
                    };

                    let response_bytes = response.encode_to_vec();

                    if let Some(reply) = msg.reply {
                        if let Err(e) = client.publish(reply, response_bytes.into()).await {
                            error!("Failed to send response: {e}");
                        }
                    }
                }
                Err(handlers_inner::HandlerError::ValidationError(error_msg)) => {
                    warn!("Validation error deleting product: {error_msg}");
                    let response = ProductDeleteResponse {
                        status: Some(catalog_messages::Status {
                            code: catalog_messages::Code::InvalidArgument.into(),
                            message: error_msg,
                            details: vec![],
                        }),
                    };
                    let response_bytes = response.encode_to_vec();
                    if let Some(reply) = msg.reply {
                        if let Err(e) = client.publish(reply, response_bytes.into()).await {
                            error!("Failed to send error response: {e}");
                        }
                    }
                }
                Err(handlers_inner::HandlerError::InternalError(error_msg)) => {
                    error!("Error deleting product: {error_msg}");
                    let response = ProductDeleteResponse {
                        status: Some(catalog_messages::Status {
                            code: catalog_messages::Code::Internal.into(),
                            message: "Internal server error".to_string(),
                            details: vec![],
                        }),
                    };

                    let response_bytes = response.encode_to_vec();

                    if let Some(reply) = msg.reply {
                        if let Err(e) = client.publish(reply, response_bytes.into()).await {
                            error!("Failed to send error response: {e}");
                        }
                    }
                }
            }
        }
        Err(err) => {
            warn!("Invalid product delete request format: {err:?}");
            let response = ProductDeleteResponse {
                status: Some(catalog_messages::Status {
                    code: catalog_messages::Code::InvalidArgument.into(),
                    message: "Invalid request format".to_string(),
                    details: vec![],
                }),
            };

            let response_bytes = response.encode_to_vec();

            if let Some(reply) = msg.reply {
                if let Err(e) = client.publish(reply, response_bytes.into()).await {
                    error!("Failed to send error response: {e}");
                }
            }
        }
    }

    Ok(())
}

pub async fn search_products(
    product_dao: Arc<dyn ProductDao + Send + Sync>,
    client: Client,
    msg: Message,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    debug!("Processing search_products request");

    let request = ProductSearchRequest::decode(&*msg.payload);
    match request {
        Ok(request) => {
            let result = handlers_inner::search_products(
                request.query,
                request.categories,
                request.brand,
                request.limit.map(|l| l as i64),
                request.offset.map(|o| o as u64),
                product_dao.as_ref(),
            )
            .await;

            match result {
                Ok(products) => {
                    let response = ProductSearchResponse {
                        products: products
                            .iter()
                            .map(|p| map_model_product_to_proto_product(p.clone()))
                            .collect(),
                        total_count: products.len() as i32,
                        status: Some(catalog_messages::Status {
                            code: catalog_messages::Code::Ok.into(),
                            message: "Products retrieved successfully".to_string(),
                            details: vec![],
                        }),
                    };

                    let response_bytes = response.encode_to_vec();

                    if let Some(reply) = msg.reply {
                        if let Err(e) = client.publish(reply, response_bytes.into()).await {
                            error!("Failed to send response: {e}");
                        }
                    }
                }
                Err(handlers_inner::HandlerError::ValidationError(error_msg)) => {
                    warn!("Validation error searching products: {error_msg}");
                    let response = ProductSearchResponse {
                        products: vec![],
                        total_count: 0,
                        status: Some(catalog_messages::Status {
                            code: catalog_messages::Code::InvalidArgument.into(),
                            message: error_msg,
                            details: vec![],
                        }),
                    };
                    let response_bytes = response.encode_to_vec();
                    if let Some(reply) = msg.reply {
                        if let Err(e) = client.publish(reply, response_bytes.into()).await {
                            error!("Failed to send error response: {e}");
                        }
                    }
                }
                Err(handlers_inner::HandlerError::InternalError(error_msg)) => {
                    error!("Error searching products: {error_msg}");
                    let response = ProductSearchResponse {
                        products: vec![],
                        total_count: 0,
                        status: Some(catalog_messages::Status {
                            code: catalog_messages::Code::Internal.into(),
                            message: "Internal server error".to_string(),
                            details: vec![],
                        }),
                    };

                    let response_bytes = response.encode_to_vec();

                    if let Some(reply) = msg.reply {
                        if let Err(e) = client.publish(reply, response_bytes.into()).await {
                            error!("Failed to send error response: {e}");
                        }
                    }
                }
            }
        }
        Err(err) => {
            warn!("Invalid product search request format: {err:?}");
            let response = ProductSearchResponse {
                products: vec![],
                total_count: 0,
                status: Some(catalog_messages::Status {
                    code: catalog_messages::Code::InvalidArgument.into(),
                    message: "Invalid request format".to_string(),
                    details: vec![],
                }),
            };

            let response_bytes = response.encode_to_vec();

            if let Some(reply) = msg.reply {
                if let Err(e) = client.publish(reply, response_bytes.into()).await {
                    error!("Failed to send error response: {e}");
                }
            }
        }
    }

    Ok(())
}

pub async fn export_products(
    product_dao: Arc<dyn ProductDao + Send + Sync>,
    client: Client,
    msg: Message,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    debug!("Processing export_products request");

    let request = ProductExportRequest::decode(&*msg.payload);
    match request {
        Ok(request) => {
            let result = handlers_inner::export_products(
                request.batch_size.map(|b| b as i64),
                request.offset.map(|o| o as u64),
                product_dao.as_ref(),
            )
            .await;

            match result {
                Ok(products) => {
                    let response = ProductExportResponse {
                        products: products
                            .iter()
                            .map(|p| map_model_product_to_proto_product(p.clone()))
                            .collect(),
                        total_count: products.len() as i32,
                        status: Some(catalog_messages::Status {
                            code: catalog_messages::Code::Ok.into(),
                            message: "Products exported successfully".to_string(),
                            details: vec![],
                        }),
                    };

                    let response_bytes = response.encode_to_vec();

                    if let Some(reply) = msg.reply {
                        if let Err(e) = client.publish(reply, response_bytes.into()).await {
                            error!("Failed to send response: {e}");
                        }
                    }
                }
                Err(handlers_inner::HandlerError::ValidationError(error_msg)) => {
                    warn!("Validation error exporting products: {error_msg}");
                    let response = ProductExportResponse {
                        products: vec![],
                        total_count: 0,
                        status: Some(catalog_messages::Status {
                            code: catalog_messages::Code::InvalidArgument.into(),
                            message: error_msg,
                            details: vec![],
                        }),
                    };
                    let response_bytes = response.encode_to_vec();
                    if let Some(reply) = msg.reply {
                        if let Err(e) = client.publish(reply, response_bytes.into()).await {
                            error!("Failed to send error response: {e}");
                        }
                    }
                }
                Err(handlers_inner::HandlerError::InternalError(error_msg)) => {
                    error!("Error exporting products: {error_msg}");
                    let response = ProductExportResponse {
                        products: vec![],
                        total_count: 0,
                        status: Some(catalog_messages::Status {
                            code: catalog_messages::Code::Internal.into(),
                            message: "Internal server error".to_string(),
                            details: vec![],
                        }),
                    };

                    let response_bytes = response.encode_to_vec();

                    if let Some(reply) = msg.reply {
                        if let Err(e) = client.publish(reply, response_bytes.into()).await {
                            error!("Failed to send error response: {e}");
                        }
                    }
                }
            }
        }
        Err(err) => {
            warn!("Invalid product export request format: {err:?}");
            let response = ProductExportResponse {
                products: vec![],
                total_count: 0,
                status: Some(catalog_messages::Status {
                    code: catalog_messages::Code::InvalidArgument.into(),
                    message: "Invalid request format".to_string(),
                    details: vec![],
                }),
            };

            let response_bytes = response.encode_to_vec();

            if let Some(reply) = msg.reply {
                if let Err(e) = client.publish(reply, response_bytes.into()).await {
                    error!("Failed to send error response: {e}");
                }
            }
        }
    }

    Ok(())
}

pub async fn get_product_slugs(
    product_dao: Arc<dyn ProductDao + Send + Sync>,
    client: Client,
    msg: Message,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    debug!("Processing get_product_slugs request");

    let request = GetProductSlugsRequest::decode(&*msg.payload);
    match request {
        Ok(request) => {
            let result = handlers_inner::get_product_slugs(
                request.batch_size,
                request.cursor,
                request.include_inactive,
                product_dao.as_ref(),
            )
            .await;

            match result {
                Ok((slugs, next_cursor, has_more)) => {
                    let response = GetProductSlugsResponse {
                        slugs: slugs.clone(),
                        next_cursor,
                        total_count: slugs.len() as i32, // Number of items in this batch
                        has_more,
                        status: Some(catalog_messages::Status {
                            code: catalog_messages::Code::Ok.into(),
                            message: "Product slugs retrieved successfully".to_string(),
                            details: vec![],
                        }),
                    };

                    let response_bytes = response.encode_to_vec();

                    if let Some(reply) = msg.reply {
                        if let Err(e) = client.publish(reply, response_bytes.into()).await {
                            error!("Failed to send response: {e}");
                        }
                    }
                }
                Err(err) => {
                    error!("Error in get_product_slugs handler: {err:?}");
                    let response = GetProductSlugsResponse {
                        slugs: vec![],
                        next_cursor: None,
                        total_count: 0,
                        has_more: false,
                        status: Some(catalog_messages::Status {
                            code: catalog_messages::Code::Internal.into(),
                            message: "Internal server error".to_string(),
                            details: vec![],
                        }),
                    };

                    let response_bytes = response.encode_to_vec();

                    if let Some(reply) = msg.reply {
                        if let Err(e) = client.publish(reply, response_bytes.into()).await {
                            error!("Failed to send error response: {e}");
                        }
                    }
                }
            }
        }
        Err(err) => {
            warn!("Invalid get product slugs request format: {err:?}");
            let response = GetProductSlugsResponse {
                slugs: vec![],
                next_cursor: None,
                total_count: 0,
                has_more: false,
                status: Some(catalog_messages::Status {
                    code: catalog_messages::Code::InvalidArgument.into(),
                    message: "Invalid request format".to_string(),
                    details: vec![],
                }),
            };

            let response_bytes = response.encode_to_vec();

            if let Some(reply) = msg.reply {
                if let Err(e) = client.publish(reply, response_bytes.into()).await {
                    error!("Failed to send error response: {e}");
                }
            }
        }
    }

    Ok(())
}

// Mapping functions to convert between domain models and protobuf messages
fn map_model_product_to_proto_product(product: Product) -> catalog_messages::Product {
    catalog_messages::Product {
        id: product.id,
        name: product.name,
        long_description: product.long_description,
        brand: product.brand,
        slug: product.slug,
        product_ref: product.product_ref,
        product_type: product.product_type,
        seo_title: product.seo_title,
        seo_description: product.seo_description,
        seo_keywords: product.seo_keywords,
        display_on_site: product.display_on_site,
        tax_code: product.tax_code,
        related_products: product.related_products,
        reviews: product.reviews.map(|r| catalog_messages::Reviews {
            bayesian_avg: r.bayesian_avg.into(),
            count: r.count,
            rating: r.rating,
        }),
        hierarchical_categories: product.hierarchical_categories.map(|hc| {
            catalog_messages::HierarchicalCategories {
                lvl0: hc.lvl0,
                lvl1: hc.lvl1,
                lvl2: hc.lvl2,
            }
        }),
        list_categories: product.list_categories,
        created_at: product.created_at.map(|dt| prost_types::Timestamp {
            seconds: dt.timestamp(),
            nanos: dt.timestamp_subsec_nanos() as i32,
        }),
        updated_at: product.updated_at.map(|dt| prost_types::Timestamp {
            seconds: dt.timestamp(),
            nanos: dt.timestamp_subsec_nanos() as i32,
        }),
        created_by: product.created_by,
        updated_by: product.updated_by,
        defining_attributes: product.defining_attributes,
        descriptive_attributes: product.descriptive_attributes,
        default_variant: product.default_variant,
        variants: product
            .variants
            .into_iter()
            .map(|v| catalog_messages::ProductVariant {
                sku: v.sku,
                defining_attributes: v.defining_attributes.unwrap_or_default(),
                abbreviated_color: v.abbreviated_color,
                abbreviated_size: v.abbreviated_size,
                height: v.height,
                width: v.width,
                length: v.length,
                weight: v.weight,
                weight_unit: v.weight_unit,
                packaging: v.packaging.map(|p| catalog_messages::Packaging {
                    height: p.height,
                    width: p.width,
                    length: p.length,
                    weight: p.weight,
                    weight_unit: p.weight_unit,
                }),
                image_urls: v.image_urls,
            })
            .collect(),
    }
}
