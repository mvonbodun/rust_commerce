use std::sync::Arc;

use async_nats::{Client, Message};
use log::{debug, error, warn};
use prost::Message as ProstMessage;

use crate::{
    catalog_messages::{
        self, CategoryExportRequest, CategoryExportResponse, CategoryImportRequest,
        CategoryImportResponse, CategoryResponse, CategoryTreeRequest, CategoryTreeResponse,
        CreateCategoryRequest, DeleteCategoryRequest, GetCategoryBySlugRequest,
        GetCategoryBySlugResponse, GetCategoryRequest, GetCategoryResponse, UpdateCategoryRequest,
    },
    AppState,
};

pub async fn create_category(
    app_state: Arc<AppState>,
    client: Client,
    msg: Message,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    debug!("Processing create_category request");

    let request = CreateCategoryRequest::decode(&*msg.payload);
    match request {
        Ok(request) => {
            let result = app_state.category_service.create_category(request).await;

            match result {
                Ok(category_response) => {
                    debug!("Category created successfully: {category_response:?}");
                    let response_bytes = category_response.encode_to_vec();

                    if let Some(reply) = msg.reply {
                        if let Err(e) = client.publish(reply, response_bytes.into()).await {
                            error!("Failed to send response: {e}");
                        }
                    }
                }
                Err(e) => {
                    error!("Error creating category: {e}");
                    let response = CategoryResponse {
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
            warn!("Invalid category create request format: {err:?}");
            let response = CategoryResponse {
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

pub async fn get_category(
    app_state: Arc<AppState>,
    client: Client,
    msg: Message,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    debug!("Processing get_category request");

    let request = GetCategoryRequest::decode(&*msg.payload);
    match request {
        Ok(request) => {
            let result = app_state.category_service.get_category(&request.id).await;

            match result {
                Ok(Some(category)) => {
                    let response = GetCategoryResponse {
                        category: Some(category),
                        status: Some(catalog_messages::Status {
                            code: catalog_messages::Code::Ok.into(),
                            message: "Category retrieved successfully".to_string(),
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
                    let response = GetCategoryResponse {
                        category: None,
                        status: Some(catalog_messages::Status {
                            code: catalog_messages::Code::NotFound.into(),
                            message: "Category not found".to_string(),
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
                Err(e) => {
                    error!("Error getting category: {e}");
                    let response = GetCategoryResponse {
                        category: None,
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
            warn!("Invalid category get request format: {err:?}");
            let response = GetCategoryResponse {
                category: None,
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

pub async fn get_category_by_slug(
    app_state: Arc<AppState>,
    client: Client,
    msg: Message,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    debug!("Processing get_category_by_slug request");

    let request = GetCategoryBySlugRequest::decode(&*msg.payload);
    match request {
        Ok(request) => {
            let result = app_state
                .category_service
                .get_category_by_slug(&request.slug)
                .await;

            match result {
                Ok(Some(category)) => {
                    let response = GetCategoryBySlugResponse {
                        category: Some(category),
                        status: Some(catalog_messages::Status {
                            code: catalog_messages::Code::Ok.into(),
                            message: "Category retrieved successfully".to_string(),
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
                    let response = GetCategoryBySlugResponse {
                        category: None,
                        status: Some(catalog_messages::Status {
                            code: catalog_messages::Code::NotFound.into(),
                            message: "Category not found".to_string(),
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
                Err(e) => {
                    error!("Error getting category by slug: {e}");
                    let response = GetCategoryBySlugResponse {
                        category: None,
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
            warn!("Invalid category get by slug request format: {err:?}");
            let response = GetCategoryBySlugResponse {
                category: None,
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

pub async fn update_category(
    app_state: Arc<AppState>,
    client: Client,
    msg: Message,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    debug!("Processing update_category request");

    let request = UpdateCategoryRequest::decode(&*msg.payload);
    match request {
        Ok(request) => {
            let result = app_state.category_service.update_category(request).await;

            match result {
                Ok(category_response) => {
                    debug!("Category updated successfully: {category_response:?}");
                    let response_bytes = category_response.encode_to_vec();

                    if let Some(reply) = msg.reply {
                        if let Err(e) = client.publish(reply, response_bytes.into()).await {
                            error!("Failed to send response: {e}");
                        }
                    }
                }
                Err(e) => {
                    error!("Error updating category: {e}");
                    let response = CategoryResponse {
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
            warn!("Invalid category update request format: {err:?}");
            let response = CategoryResponse {
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

pub async fn delete_category(
    app_state: Arc<AppState>,
    client: Client,
    msg: Message,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    debug!("Processing delete_category request");

    let request = DeleteCategoryRequest::decode(&*msg.payload);
    match request {
        Ok(request) => {
            let result = app_state
                .category_service
                .delete_category(&request.id)
                .await;

            match result {
                Ok(_success) => {
                    let response = CategoryResponse {
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
                    let response_bytes = response.encode_to_vec();

                    if let Some(reply) = msg.reply {
                        if let Err(e) = client.publish(reply, response_bytes.into()).await {
                            error!("Failed to send response: {e}");
                        }
                    }
                }
                Err(e) => {
                    error!("Error deleting category: {e}");
                    let response = CategoryResponse {
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
            warn!("Invalid category delete request format: {err:?}");
            let response = CategoryResponse {
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

pub async fn export_categories(
    app_state: Arc<AppState>,
    client: Client,
    msg: Message,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    debug!("Processing export_categories request");

    let request = CategoryExportRequest::decode(&*msg.payload);
    match request {
        Ok(request) => {
            // The export_categories method takes batch_size and offset, not include_inactive and parent_id
            let result = app_state
                .category_service
                .export_categories(
                    request.batch_size.map(|b| b as i64),
                    request.offset.map(|o| o as u64),
                )
                .await;

            match result {
                Ok(categories) => {
                    let response = CategoryExportResponse {
                        categories,
                        status: Some(catalog_messages::Status {
                            code: catalog_messages::Code::Ok.into(),
                            message: "Categories exported successfully".to_string(),
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
                Err(e) => {
                    error!("Error exporting categories: {e}");
                    let response = CategoryExportResponse {
                        categories: vec![],
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
            warn!("Invalid category export request format: {err:?}");
            let response = CategoryExportResponse {
                categories: vec![],
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

pub async fn import_categories(
    app_state: Arc<AppState>,
    client: Client,
    msg: Message,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    debug!("Processing import_categories request");

    let request = CategoryImportRequest::decode(&*msg.payload);
    match request {
        Ok(request) => {
            // The import_categories method takes a second boolean parameter for dry_run
            let dry_run = request.dry_run;
            let result = app_state
                .category_service
                .import_categories(request.categories, dry_run)
                .await;

            match result {
                Ok(import_result) => {
                    let response = CategoryImportResponse {
                        successful_imports: import_result.successful_imports as i32,
                        failed_imports: import_result.failed_imports as i32,
                        total_processed: import_result.total_processed as i32,
                        errors: import_result.errors,
                        status: Some(catalog_messages::Status {
                            code: catalog_messages::Code::Ok.into(),
                            message: format!(
                                "Import completed: {} successful, {} failed",
                                import_result.successful_imports, import_result.failed_imports
                            ),
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
                Err(e) => {
                    error!("Error importing categories: {e}");
                    let response = CategoryImportResponse {
                        successful_imports: 0,
                        failed_imports: 0,
                        total_processed: 0,
                        errors: vec![format!("Import failed: {e}")],
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
            warn!("Invalid category import request format: {err:?}");
            let response = CategoryImportResponse {
                successful_imports: 0,
                failed_imports: 0,
                total_processed: 0,
                errors: vec!["Invalid request format".to_string()],
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

pub async fn get_category_tree(
    app_state: Arc<AppState>,
    client: Client,
    msg: Message,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    debug!("Processing get_category_tree request");

    let request = CategoryTreeRequest::decode(&*msg.payload);
    match request {
        Ok(request) => {
            // The get_category_tree method takes max_depth, include_inactive, and rebuild_cache
            let result = app_state
                .category_service
                .get_category_tree(
                    request.max_depth,
                    request.include_inactive,
                    request.rebuild_cache,
                )
                .await;

            match result {
                Ok(tree_nodes) => {
                    let response = CategoryTreeResponse {
                        tree: tree_nodes,
                        status: Some(catalog_messages::Status {
                            code: catalog_messages::Code::Ok.into(),
                            message: "Category tree retrieved successfully".to_string(),
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
                Err(e) => {
                    error!("Error getting category tree: {e}");
                    let response = CategoryTreeResponse {
                        tree: vec![],
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
            warn!("Invalid category tree request format: {err:?}");
            let response = CategoryTreeResponse {
                tree: vec![],
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
