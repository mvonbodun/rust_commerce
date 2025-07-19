use std::{collections::HashMap, future::Future, pin::Pin, sync::Arc};
use async_nats::{Client, Message};
use log::{debug, error};
use prost::Message as ProstMessage;
use crate::{
    catalog_messages::{self, ProductCreateRequest, ProductCreateResponse, ProductGetRequest, ProductGetResponse, ProductUpdateRequest, ProductUpdateResponse, ProductDeleteRequest, ProductDeleteResponse, ProductSearchRequest, ProductSearchResponse},
    persistence::product_dao::ProductDao,
};

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
    
    // Decode the request
    let request = ProductCreateRequest::decode(&*msg.payload)?;
    debug!("Decoded create_product request: {:?}", request);
    
    // TODO: Convert protobuf request to domain model and call DAO
    // For now, just return a placeholder response
    
    let response = ProductCreateResponse {
        product: None,
        status: Some(catalog_messages::Status {
            code: 12, // UNIMPLEMENTED
            message: "create_product not yet implemented".to_string(),
            details: vec![],
        }),
    };
    
    let response_bytes = response.encode_to_vec();
    
    if let Some(reply) = msg.reply {
        if let Err(e) = client.publish(reply, response_bytes.into()).await {
            error!("Failed to send response: {}", e);
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
    
    // Decode the request
    let request = ProductGetRequest::decode(&*msg.payload)?;
    debug!("Decoded get_product request: {:?}", request);
    
    // TODO: Call DAO and convert domain model to protobuf response
    // For now, just return a placeholder response
    
    let response = ProductGetResponse {
        product: None,
        status: Some(catalog_messages::Status {
            code: 12, // UNIMPLEMENTED
            message: "get_product not yet implemented".to_string(),
            details: vec![],
        }),
    };
    
    let response_bytes = response.encode_to_vec();
    
    if let Some(reply) = msg.reply {
        if let Err(e) = client.publish(reply, response_bytes.into()).await {
            error!("Failed to send response: {}", e);
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
    
    // Decode the request
    let request = ProductUpdateRequest::decode(&*msg.payload)?;
    debug!("Decoded update_product request: {:?}", request);
    
    // TODO: Convert protobuf request to domain model and call DAO
    // For now, just return a placeholder response
    
    let response = ProductUpdateResponse {
        product: None,
        status: Some(catalog_messages::Status {
            code: 12, // UNIMPLEMENTED
            message: "update_product not yet implemented".to_string(),
            details: vec![],
        }),
    };
    
    let response_bytes = response.encode_to_vec();
    
    if let Some(reply) = msg.reply {
        if let Err(e) = client.publish(reply, response_bytes.into()).await {
            error!("Failed to send response: {}", e);
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
    
    // Decode the request
    let request = ProductDeleteRequest::decode(&*msg.payload)?;
    debug!("Decoded delete_product request: {:?}", request);
    
    // TODO: Call DAO to delete product
    // For now, just return a placeholder response
    
    let response = ProductDeleteResponse {
        status: Some(catalog_messages::Status {
            code: 12, // UNIMPLEMENTED
            message: "delete_product not yet implemented".to_string(),
            details: vec![],
        }),
    };
    
    let response_bytes = response.encode_to_vec();
    
    if let Some(reply) = msg.reply {
        if let Err(e) = client.publish(reply, response_bytes.into()).await {
            error!("Failed to send response: {}", e);
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
    
    // Decode the request
    let request = ProductSearchRequest::decode(&*msg.payload)?;
    debug!("Decoded search_products request: {:?}", request);
    
    // TODO: Call DAO to search products and convert results to protobuf
    // For now, just return a placeholder response
    
    let response = ProductSearchResponse {
        products: vec![],
        total_count: 0,
        status: Some(catalog_messages::Status {
            code: 12, // UNIMPLEMENTED
            message: "search_products not yet implemented".to_string(),
            details: vec![],
        }),
    };
    
    let response_bytes = response.encode_to_vec();
    
    if let Some(reply) = msg.reply {
        if let Err(e) = client.publish(reply, response_bytes.into()).await {
            error!("Failed to send response: {}", e);
        }
    }
    
    Ok(())
}
