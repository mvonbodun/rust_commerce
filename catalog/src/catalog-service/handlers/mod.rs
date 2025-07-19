use std::{collections::HashMap, future::Future, pin::Pin, sync::Arc};
use async_nats::{Client, Message};
use log::{debug, error, warn};
use prost::Message as ProstMessage;
use crate::{
    catalog_messages::{self, ProductCreateRequest, ProductCreateResponse, ProductGetRequest, ProductGetResponse, ProductUpdateRequest, ProductUpdateResponse, ProductDeleteRequest, ProductDeleteResponse, ProductSearchRequest, ProductSearchResponse},
    persistence::product_dao::ProductDao,
    model::Product,
};

mod handlers_inner;

pub type NatsFn = Box<
    dyn Fn(
            Arc<dyn ProductDao + Send + Sync>,
            Client,
            Message,
        ) -> Pin<Box<dyn Future<Output = Result<(), Box<dyn std::error::Error + Send + Sync>>> + Send>>
        + Send
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
                request.name,
                request.product_ref,
                request.brand,
                request.long_description,
                request.product_type,
                request.seo_title,
                request.seo_description,
                request.seo_keywords,
                request.display_on_site,
                request.tax_code,
                None, // created_by - could be extracted from metadata
                product_dao.as_ref(),
            ).await;
            
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
                            error!("Failed to send response: {}", e);
                        }
                    }
                }
                Err(handlers_inner::HandlerError::InternalError(error_msg)) => {
                    error!("Error creating product: {}", error_msg);
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
                            error!("Failed to send error response: {}", e);
                        }
                    }
                }
            }
        }
        Err(err) => {
            warn!("Invalid product create request format: {:?}", err);
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
                    error!("Failed to send error response: {}", e);
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
                            error!("Failed to send response: {}", e);
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
                            error!("Failed to send response: {}", e);
                        }
                    }
                }
                Err(handlers_inner::HandlerError::InternalError(error_msg)) => {
                    error!("Error getting product: {}", error_msg);
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
                            error!("Failed to send error response: {}", e);
                        }
                    }
                }
            }
        }
        Err(err) => {
            warn!("Invalid product get request format: {:?}", err);
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
                    error!("Failed to send error response: {}", e);
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
    match request {        Ok(request) => {
            if let Some(product) = request.product {
                // Convert protobuf Product to domain Product
                let domain_product = map_proto_product_to_model_product(product);
                
                let result = handlers_inner::update_product(
                    request.id,
                    domain_product,
                    product_dao.as_ref(),
                ).await;
            
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
                                error!("Failed to send response: {}", e);
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
                                error!("Failed to send response: {}", e);
                            }
                        }
                    }
                    Err(handlers_inner::HandlerError::InternalError(error_msg)) => {
                        error!("Error updating product: {}", error_msg);
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
                                error!("Failed to send error response: {}", e);
                            }
                        }
                    }
                }
            } else {
                let response = ProductUpdateResponse {
                    product: None,
                    status: Some(catalog_messages::Status {
                        code: catalog_messages::Code::InvalidArgument.into(),
                        message: "Product object is required".to_string(),
                        details: vec![],
                    }),
                };
                
                let response_bytes = response.encode_to_vec();
                
                if let Some(reply) = msg.reply {
                    if let Err(e) = client.publish(reply, response_bytes.into()).await {
                        error!("Failed to send error response: {}", e);
                    }
                }
            }
        }
        Err(err) => {
            warn!("Invalid product update request format: {:?}", err);
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
                    error!("Failed to send error response: {}", e);
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
            let result = handlers_inner::delete_product(
                request.id,
                product_dao.as_ref(),
            ).await;
            
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
                            error!("Failed to send response: {}", e);
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
                            error!("Failed to send response: {}", e);
                        }
                    }
                }
                Err(handlers_inner::HandlerError::InternalError(error_msg)) => {
                    error!("Error deleting product: {}", error_msg);
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
                            error!("Failed to send error response: {}", e);
                        }
                    }
                }
            }
        }
        Err(err) => {
            warn!("Invalid product delete request format: {:?}", err);
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
                    error!("Failed to send error response: {}", e);
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
            ).await;
            
            match result {
                Ok(products) => {
                    let response = ProductSearchResponse {
                        products: products.iter().map(|p| map_model_product_to_proto_product(p.clone())).collect(),
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
                            error!("Failed to send response: {}", e);
                        }
                    }
                }
                Err(handlers_inner::HandlerError::InternalError(error_msg)) => {
                    error!("Error searching products: {}", error_msg);
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
                            error!("Failed to send error response: {}", e);
                        }
                    }
                }
            }
        }
        Err(err) => {
            warn!("Invalid product search request format: {:?}", err);
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
                    error!("Failed to send error response: {}", e);
                }
            }
        }
    }
    
    Ok(())
}

// Mapping functions to convert between domain models and protobuf messages
fn map_proto_product_to_model_product(proto_product: catalog_messages::Product) -> Product {
    Product {
        id: proto_product.id,
        name: proto_product.name,
        long_description: proto_product.long_description,
        brand: proto_product.brand,
        slug: proto_product.slug,
        product_ref: proto_product.product_ref,
        product_type: proto_product.product_type,
        seo_title: proto_product.seo_title,
        seo_description: proto_product.seo_description,
        seo_keywords: proto_product.seo_keywords,
        display_on_site: proto_product.display_on_site,
        tax_code: proto_product.tax_code,
        related_products: proto_product.related_products,
        reviews: proto_product.reviews.map(|r| crate::model::Reviews {
            bayesian_avg: r.bayesian_avg,
            count: r.count,
            rating: r.rating,
        }),
        hierarchical_categories: proto_product.hierarchical_categories.map(|hc| crate::model::HierarchicalCategories {
            lvl0: hc.lvl0,
            lvl1: hc.lvl1,
            lvl2: hc.lvl2,
            lvl3: hc.lvl3,
        }),
        list_categories: proto_product.list_categories,
        created_at: proto_product.created_at.map(|ts| {
            chrono::DateTime::from_timestamp(ts.seconds, ts.nanos as u32).unwrap_or_default()
        }),
        updated_at: proto_product.updated_at.map(|ts| {
            chrono::DateTime::from_timestamp(ts.seconds, ts.nanos as u32).unwrap_or_default()
        }),
        created_by: proto_product.created_by,
        updated_by: proto_product.updated_by,
        defining_attributes: proto_product.defining_attributes,
        descriptive_attributes: proto_product.descriptive_attributes,
        default_variant: proto_product.default_variant,
        variants: proto_product.variants.into_iter().map(|v| crate::model::ProductVariant {
            sku: v.sku,
            defining_attributes: Some(v.defining_attributes),
            abbreviated_color: v.abbreviated_color,
            abbreviated_size: v.abbreviated_size,
            height: v.height,
            width: v.width,
            length: v.length,
            weight: v.weight,
            weight_unit: v.weight_unit,
            packaging: v.packaging.map(|p| crate::model::Packaging {
                height: p.height,
                width: p.width,
                length: p.length,
                weight: p.weight,
                weight_unit: p.weight_unit,
            }),
            image_urls: v.image_urls,
        }).collect(),
    }
}

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
            bayesian_avg: r.bayesian_avg,
            count: r.count,
            rating: r.rating,
        }),
        hierarchical_categories: product.hierarchical_categories.map(|hc| catalog_messages::HierarchicalCategories {
            lvl0: hc.lvl0,
            lvl1: hc.lvl1,
            lvl2: hc.lvl2,
            lvl3: hc.lvl3,
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
        variants: product.variants.into_iter().map(|v| catalog_messages::ProductVariant {
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
        }).collect(),
    }
}
