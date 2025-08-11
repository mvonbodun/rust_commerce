use std::sync::Arc;
use async_nats::Message;
use prost::Message as ProstMessage;
use log::{debug, error};
use crate::{
    catalog_messages::{
        CreateCategoryRequest, CategoryResponse, GetCategoryRequest, GetCategoryBySlugRequest,
        CategoryExportRequest, CategoryExportResponse, Status, Code,
    },
    handlers::category_service::CategoryService,
};

/// Handle category creation requests
pub async fn handle_create_category(
    msg: &Message,
    category_service: Arc<CategoryService>,
) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    debug!("Handling create category request");

    let request = CreateCategoryRequest::decode(&*msg.payload)?;
    debug!("Decoded request: {:?}", request);

    match category_service.create_category(request).await {
        Ok(category_response) => {
            debug!("Category created successfully: {:?}", category_response);
            Ok(category_response.encode_to_vec())
        }
        Err(e) => {
            error!("Failed to create category: {}", e);
            let error_response = CategoryResponse {
                id: String::new(),
                slug: String::new(),
                name: String::new(),
                short_description: String::new(),
                full_description: None,
                path: String::new(),
                ancestors: vec![],
                parent_id: None,
                level: 0,
                children_count: 0,
                product_count: 0,
                is_active: false,
                display_order: 0,
                seo: None,
                created_at: None,
                updated_at: None,
            };
            Ok(error_response.encode_to_vec())
        }
    }
}

/// Handle get category by ID requests
pub async fn handle_get_category(
    msg: &Message,
    category_service: Arc<CategoryService>,
) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    debug!("Handling get category request");

    let request = GetCategoryRequest::decode(&*msg.payload)?;
    debug!("Decoded request: {:?}", request);

    match category_service.get_category(&request.id).await {
        Ok(Some(category_response)) => {
            debug!("Category found: {:?}", category_response);
            Ok(category_response.encode_to_vec())
        }
        Ok(None) => {
            debug!("Category not found: {}", request.id);
            let not_found_response = CategoryResponse {
                id: String::new(),
                slug: String::new(),
                name: String::new(),
                short_description: String::new(),
                full_description: None,
                path: String::new(),
                ancestors: vec![],
                parent_id: None,
                level: 0,
                children_count: 0,
                product_count: 0,
                is_active: false,
                display_order: 0,
                seo: None,
                created_at: None,
                updated_at: None,
            };
            Ok(not_found_response.encode_to_vec())
        }
        Err(e) => {
            error!("Failed to get category: {}", e);
            let error_response = CategoryResponse {
                id: String::new(),
                slug: String::new(),
                name: String::new(),
                short_description: String::new(),
                full_description: None,
                path: String::new(),
                ancestors: vec![],
                parent_id: None,
                level: 0,
                children_count: 0,
                product_count: 0,
                is_active: false,
                display_order: 0,
                seo: None,
                created_at: None,
                updated_at: None,
            };
            Ok(error_response.encode_to_vec())
        }
    }
}

/// Handle get category by slug requests
pub async fn handle_get_category_by_slug(
    msg: &Message,
    category_service: Arc<CategoryService>,
) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    debug!("Handling get category by slug request");

    let request = GetCategoryBySlugRequest::decode(&*msg.payload)?;
    debug!("Decoded request: {:?}", request);

    match category_service.get_category_by_slug(&request.slug).await {
        Ok(Some(category_response)) => {
            debug!("Category found: {:?}", category_response);
            Ok(category_response.encode_to_vec())
        }
        Ok(None) => {
            debug!("Category not found: {}", request.slug);
            let not_found_response = CategoryResponse {
                id: String::new(),
                slug: String::new(),
                name: String::new(),
                short_description: String::new(),
                full_description: None,
                path: String::new(),
                ancestors: vec![],
                parent_id: None,
                level: 0,
                children_count: 0,
                product_count: 0,
                is_active: false,
                display_order: 0,
                seo: None,
                created_at: None,
                updated_at: None,
            };
            Ok(not_found_response.encode_to_vec())
        }
        Err(e) => {
            error!("Failed to get category by slug: {}", e);
            let error_response = CategoryResponse {
                id: String::new(),
                slug: String::new(),
                name: String::new(),
                short_description: String::new(),
                full_description: None,
                path: String::new(),
                ancestors: vec![],
                parent_id: None,
                level: 0,
                children_count: 0,
                product_count: 0,
                is_active: false,
                display_order: 0,
                seo: None,
                created_at: None,
                updated_at: None,
            };
            Ok(error_response.encode_to_vec())
        }
    }
}

/// Handle category export requests
pub async fn handle_export_categories(
    msg: &Message,
    category_service: Arc<CategoryService>,
) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    debug!("Handling export categories request");

    let request = CategoryExportRequest::decode(&*msg.payload)?;
    debug!("Decoded request: {:?}", request);

    match category_service.export_categories(request.batch_size.map(|b| b as i64), request.offset.map(|o| o as u64)).await {
        Ok(categories) => {
            debug!("Categories exported successfully, count: {}", categories.len());
            
            let response = CategoryExportResponse {
                categories,
                status: Some(Status {
                    code: Code::Ok as i32,
                    message: "Categories exported successfully".to_string(),
                    details: vec![],
                }),
            };
            
            Ok(response.encode_to_vec())
        }
        Err(e) => {
            error!("Failed to export categories: {}", e);
            
            let error_response = CategoryExportResponse {
                categories: vec![],
                status: Some(Status {
                    code: Code::Internal as i32,
                    message: format!("Failed to export categories: {}", e),
                    details: vec![],
                }),
            };
            
            Ok(error_response.encode_to_vec())
        }
    }
}
