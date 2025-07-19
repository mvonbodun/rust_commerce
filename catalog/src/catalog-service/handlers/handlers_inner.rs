use log::{debug, error};
use crate::model::{Product, ProductBuilder};
use crate::persistence::product_dao::ProductDao;

pub enum HandlerError {
    InternalError(String),
}

impl HandlerError {
    pub fn default_internal_error() -> Self {
        HandlerError::InternalError("An unexpected error occurred. Please try again".to_owned())
    }
}

pub async fn create_product(
    name: String,
    product_ref: String,
    brand: Option<String>,
    long_description: Option<String>,
    product_type: Option<String>,
    seo_title: Option<String>,
    seo_description: Option<String>,
    seo_keywords: Option<String>,
    display_on_site: bool,
    tax_code: Option<String>,
    created_by: Option<String>,
    product_dao: &(dyn ProductDao + Sync + Send),
) -> Result<Product, HandlerError> {
    debug!("Before call to create_product handler_inner");
    
    let mut product_builder = ProductBuilder::new(name, product_ref);
    
    if let Some(brand) = brand {
        product_builder.brand(brand);
    }
    
    if let Some(long_description) = long_description {
        product_builder.long_description(long_description);
    }
    
    if let Some(product_type) = product_type {
        product_builder.product_type(product_type);
    }
    
    if let Some(seo_title) = seo_title {
        product_builder.seo_title(seo_title);
    }
    
    if let Some(seo_description) = seo_description {
        product_builder.seo_description(seo_description);
    }
    
    if let Some(seo_keywords) = seo_keywords {
        product_builder.seo_keywords(seo_keywords);
    }
    
    product_builder.display_on_site(display_on_site);
    
    if let Some(tax_code) = tax_code {
        product_builder.tax_code(tax_code);
    }
    
    if let Some(created_by) = created_by {
        product_builder.created_by(created_by);
    }

    let product = product_builder.build();
    let result = product_dao.create_product(product).await;

    match result {
        Ok(product) => Ok(product),
        Err(e) => {
            error!("Error creating product: {}", e);
            Err(HandlerError::InternalError(format!(
                "Failed to create product: {}",
                e
            )))
        }
    }
}

pub async fn get_product(
    product_id: String,
    product_dao: &(dyn ProductDao + Sync + Send),
) -> Result<Option<Product>, HandlerError> {
    debug!("Before call to get_product handler_inner");
    let result = product_dao.get_product(&product_id).await;
    debug!("After call to get_product handler_inner: {:?}", result);

    match result {
        Ok(Some(product)) => Ok(Some(product)),
        Ok(None) => Ok(None),
        Err(e) => {
            error!("Error getting product: {}", e);
            Err(HandlerError::InternalError(format!(
                "Failed to get product: {}",
                e
            )))
        }
    }
}

pub async fn update_product(
    product_id: String,
    product: Product,
    product_dao: &(dyn ProductDao + Send + Sync),
) -> Result<Option<Product>, HandlerError> {
    debug!("Before call to update_product handler_inner");
    let result = product_dao.update_product(&product_id, product).await;

    match result {
        Ok(Some(product)) => Ok(Some(product)),
        Ok(None) => Ok(None),
        Err(e) => {
            error!("Error updating product: {}", e);
            Err(HandlerError::InternalError(format!(
                "Failed to update product: {}",
                e
            )))
        }
    }
}

pub async fn delete_product(
    product_id: String,
    product_dao: &(dyn ProductDao + Send + Sync),
) -> Result<bool, HandlerError> {
    debug!("Before call to delete_product handler_inner");
    let result = product_dao.delete_product(&product_id).await;

    match result {
        Ok(deleted) => Ok(deleted),
        Err(e) => {
            error!("Error deleting product: {}", e);
            Err(HandlerError::InternalError(format!(
                "Failed to delete product: {}",
                e
            )))
        }
    }
}

pub async fn search_products(
    query: Option<String>,
    categories: Vec<String>,
    brand: Option<String>,
    limit: Option<i64>,
    offset: Option<u64>,
    product_dao: &(dyn ProductDao + Send + Sync),
) -> Result<Vec<Product>, HandlerError> {
    debug!("Before call to search_products handler_inner");
    let result = product_dao.search_products(
        query.as_deref(),
        &categories,
        brand.as_deref(),
        limit,
        offset,
    ).await;

    match result {
        Ok(products) => Ok(products),
        Err(e) => {
            error!("Error searching products: {}", e);
            Err(HandlerError::InternalError(format!(
                "Failed to search products: {}",
                e
            )))
        }
    }
}
