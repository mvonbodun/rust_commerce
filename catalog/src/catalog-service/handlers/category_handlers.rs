use std::sync::Arc;
use async_nats::Message;
use prost::Message as ProstMessage;
use log::{debug, error};
use crate::{
    catalog_messages::{
        CreateCategoryRequest, CategoryResponse, GetCategoryRequest, GetCategoryBySlugRequest,
        CategoryExportRequest, CategoryExportResponse, UpdateCategoryRequest, DeleteCategoryRequest,
        CategoryImportRequest, CategoryImportResponse, CategoryTreeRequest, CategoryTreeResponse,
        Status, Code,
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

/// Handle update category requests
pub async fn handle_update_category(
    msg: &Message,
    category_service: Arc<CategoryService>,
) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    debug!("Handling update category request");

    let request = UpdateCategoryRequest::decode(&*msg.payload)?;
    debug!("Decoded request: {:?}", request);

    match category_service.update_category(request).await {
        Ok(category_response) => {
            debug!("Category updated successfully: {:?}", category_response);
            Ok(category_response.encode_to_vec())
        }
        Err(e) => {
            error!("Failed to update category: {}", e);
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

/// Handle delete category requests
pub async fn handle_delete_category(
    msg: &Message,
    category_service: Arc<CategoryService>,
) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    debug!("Handling delete category request");

    let request = DeleteCategoryRequest::decode(&*msg.payload)?;
    debug!("Decoded request: {:?}", request);

    match category_service.delete_category(&request.id).await {
        Ok(success) => {
            debug!("Category delete result: {}", success);
            
            let response = Status {
                code: if success { Code::Ok as i32 } else { Code::NotFound as i32 },
                message: if success { 
                    "Category deleted successfully".to_string() 
                } else { 
                    "Category not found".to_string() 
                },
                details: vec![],
            };
            
            Ok(response.encode_to_vec())
        }
        Err(e) => {
            error!("Failed to delete category: {}", e);
            
            let error_response = Status {
                code: Code::Internal as i32,
                message: format!("Failed to delete category: {}", e),
                details: vec![],
            };
            
            Ok(error_response.encode_to_vec())
        }
    }
}

/// Handle import categories requests
pub async fn handle_import_categories(
    msg: &Message,
    category_service: Arc<CategoryService>,
) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    debug!("Handling import categories request");

    let request = CategoryImportRequest::decode(&*msg.payload)?;
    debug!("Decoded request with {} categories", request.categories.len());

    match category_service.import_categories(request.categories, request.dry_run).await {
        Ok(import_result) => {
            debug!("Categories import completed: {:?}", import_result);
            
            let response = CategoryImportResponse {
                successful_imports: import_result.successful_imports as i32,
                failed_imports: import_result.failed_imports as i32,
                total_processed: import_result.total_processed as i32,
                errors: import_result.errors,
                status: Some(Status {
                    code: Code::Ok as i32,
                    message: "Import completed".to_string(),
                    details: vec![],
                }),
            };
            
            Ok(response.encode_to_vec())
        }
        Err(e) => {
            error!("Failed to import categories: {}", e);
            
            let error_response = CategoryImportResponse {
                successful_imports: 0,
                failed_imports: 0,
                total_processed: 0,
                errors: vec![format!("Import failed: {}", e)],
                status: Some(Status {
                    code: Code::Internal as i32,
                    message: format!("Failed to import categories: {}", e),
                    details: vec![],
                }),
            };
            
            Ok(error_response.encode_to_vec())
        }
    }
}

/// Handle category tree retrieval requests
pub async fn handle_get_category_tree(
    msg: &Message,
    category_service: Arc<CategoryService>,
) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    debug!("Handling get category tree request");

    let request = CategoryTreeRequest::decode(&*msg.payload)?;
    debug!("Decoded request: max_depth={:?}, include_inactive={:?}, rebuild_cache={:?}", 
           request.max_depth, request.include_inactive, request.rebuild_cache);

    match category_service.get_category_tree(
        request.max_depth, 
        request.include_inactive, 
        request.rebuild_cache
    ).await {
        Ok(tree_nodes) => {
            debug!("Category tree retrieved successfully with {} root nodes", tree_nodes.len());
            
            let response = CategoryTreeResponse {
                tree: tree_nodes,
                status: Some(Status {
                    code: Code::Ok as i32,
                    message: "Category tree retrieved successfully".to_string(),
                    details: vec![],
                }),
            };
            
            Ok(response.encode_to_vec())
        }
        Err(e) => {
            error!("Failed to get category tree: {}", e);
            
            let error_response = CategoryTreeResponse {
                tree: vec![],
                status: Some(Status {
                    code: Code::Internal as i32,
                    message: format!("Failed to get category tree: {}", e),
                    details: vec![],
                }),
            };
            
            Ok(error_response.encode_to_vec())
        }
    }
}
