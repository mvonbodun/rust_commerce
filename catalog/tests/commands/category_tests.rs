use crate::helpers::{self, *};
use crate::helpers::catalog_messages::*;
use prost::Message;
use rust_common::test_helpers::{fixtures, TestApp};

// ============================================================================
// CATEGORY CREATE TESTS
// ============================================================================

#[tokio::test]
async fn test_category_create_root_category() {
    let app = TestApp::spawn().await;
    let builder = fixtures::category::CategoryBuilder::root();
    
    let request = CreateCategoryRequest {
        name: builder.name.clone(),
        slug: builder.slug.clone(),
        short_description: builder.short_description.clone(),
        full_description: None,
        parent_id: None,
        display_order: builder.display_order,
        seo: None,
        is_active: Some(true),
        parent_slug: None,
    };
    
    let response = app
        .request("catalog.create_category", request.encode_to_vec())
        .await
        .expect("Request should succeed");
    
    let category = CategoryResponse::decode(&*response.payload)
        .expect("Response should decode");
    
    // CategoryResponse IS the category itself
    assert!(!category.id.is_empty());
    assert_eq!(category.name, builder.name);
    assert_eq!(category.slug, builder.slug);
}

#[tokio::test]
async fn test_category_create_child_category() {
    let app = TestApp::spawn().await;
    
    // Create parent category first
    let parent_builder = fixtures::category::CategoryBuilder::root();
    let parent_id = create_test_category(&app, parent_builder).await
        .expect("Should create parent category");
    
    // Create child category
    let child_builder = fixtures::category::CategoryBuilder::child_of(parent_id.clone());
    
    let request = CreateCategoryRequest {
        name: child_builder.name.clone(),
        slug: child_builder.slug.clone(),
        short_description: child_builder.short_description.clone(),
        full_description: None,
        parent_id: child_builder.parent_id.clone(),
        display_order: child_builder.display_order,
        seo: None,
        is_active: Some(true),
        parent_slug: None,
    };
    
    let response = app
        .request("catalog.create_category", request.encode_to_vec())
        .await
        .expect("Request should succeed");
    
    let category = CategoryResponse::decode(&*response.payload)
        .expect("Response should decode");
    
    assert_eq!(category.parent_id, Some(parent_id));
}

// ============================================================================
// CATEGORY GET TESTS
// ============================================================================

#[tokio::test]
async fn test_category_get_existing() {
    let app = TestApp::spawn().await;
    
    // Create a category
    let builder = fixtures::category::CategoryBuilder::default();
    let category_id = create_test_category(&app, builder.clone()).await
        .expect("Should create category");
    
    let request = GetCategoryRequest {
        id: category_id.clone(),
    };
    
    let response = app
        .request("catalog.get_category", request.encode_to_vec())
        .await
        .expect("Request should succeed");
    
    let category = CategoryResponse::decode(&*response.payload)
        .expect("Response should decode");
    
    assert_eq!(category.id, category_id);
    assert_eq!(category.name, builder.name);
}

#[tokio::test]
async fn test_category_get_non_existent() {
    let app = TestApp::spawn().await;
    
    let request = GetCategoryRequest {
        id: "non-existent-id".to_string(),
    };
    
    let response = app
        .request("catalog.get_category", request.encode_to_vec())
        .await;
    
    // Should fail for non-existent category
    assert!(response.is_err() || {
        if let Ok(msg) = response {
            CategoryResponse::decode(&*msg.payload).is_err()
        } else {
            false
        }
    });
}

// ============================================================================
// CATEGORY GET BY SLUG TESTS
// ============================================================================

#[tokio::test]
async fn test_category_get_by_slug_existing() {
    let app = TestApp::spawn().await;
    
    let slug = fixtures::valid_slug();
    let mut builder = fixtures::category::CategoryBuilder::default();
    builder.slug = slug.clone();
    
    let category_id = create_test_category(&app, builder).await
        .expect("Should create category");
    
    let request = GetCategoryBySlugRequest {
        slug: slug.clone(),
    };
    
    let response = app
        .request("catalog.get_category_by_slug", request.encode_to_vec())
        .await
        .expect("Request should succeed");
    
    let category = CategoryResponse::decode(&*response.payload)
        .expect("Response should decode");
    
    assert_eq!(category.id, category_id);
    assert_eq!(category.slug, slug);
}

// ============================================================================
// CATEGORY UPDATE TESTS
// ============================================================================

#[tokio::test]
async fn test_category_update_name_and_description() {
    let app = TestApp::spawn().await;
    
    // Create a category
    let builder = fixtures::category::CategoryBuilder::default();
    let category_id = create_test_category(&app, builder).await
        .expect("Should create category");
    
    let request = UpdateCategoryRequest {
        id: category_id.clone(),
        name: Some("Updated Name".to_string()),
        slug: None,
        short_description: Some("Updated short description".to_string()),
        full_description: Some("Updated full description".to_string()),
        display_order: None,
        seo: None,
        is_active: None,
    };
    
    let response = app
        .request("catalog.update_category", request.encode_to_vec())
        .await
        .expect("Request should succeed");
    
    let category = CategoryResponse::decode(&*response.payload)
        .expect("Response should decode");
    
    assert_eq!(category.name, "Updated Name");
    assert_eq!(category.short_description, "Updated short description");
}

// ============================================================================
// CATEGORY DELETE TESTS
// ============================================================================

#[tokio::test]
async fn test_category_delete_existing() {
    let app = TestApp::spawn().await;
    
    // Create a category
    let builder = fixtures::category::CategoryBuilder::default();
    let category_id = create_test_category(&app, builder).await
        .expect("Should create category");
    
    let request = DeleteCategoryRequest {
        id: category_id.clone(),
    };
    
    let response = app
        .request("catalog.delete_category", request.encode_to_vec())
        .await
        .expect("Request should succeed");
    
    // Delete returns common.Status
    let status = shared_proto::common::Status::decode(&*response.payload)
        .expect("Response should decode");
    
    assert_eq!(status.code, Code::Ok as i32);
    
    // Verify it's deleted by trying to get it
    let get_request = GetCategoryRequest {
        id: category_id,
    };
    
    let get_response = app
        .request("catalog.get_category", get_request.encode_to_vec())
        .await;
    
    assert!(get_response.is_err() || {
        if let Ok(msg) = get_response {
            CategoryResponse::decode(&*msg.payload).is_err()
        } else {
            false
        }
    });
}

// ============================================================================
// CATEGORY TREE TESTS
// ============================================================================

#[tokio::test]
async fn test_category_tree_with_hierarchy() {
    let app = TestApp::spawn().await;
    
    // Create a hierarchy: Root -> Child1, Child2
    let root = fixtures::category::CategoryBuilder::root();
    let root_id = create_test_category(&app, root).await
        .expect("Should create root");
    
    let child1 = fixtures::category::CategoryBuilder::child_of(root_id.clone());
    create_test_category(&app, child1).await
        .expect("Should create child1");
    
    let child2 = fixtures::category::CategoryBuilder::child_of(root_id.clone());
    create_test_category(&app, child2).await
        .expect("Should create child2");
    
    // Get tree
    let request = CategoryTreeRequest {
        max_depth: Some(10),
        include_inactive: None,
        rebuild_cache: None,
    };
    
    let response = app
        .request("catalog.get_category_tree", request.encode_to_vec())
        .await
        .expect("Request should succeed");
    
    let tree_response = CategoryTreeResponse::decode(&*response.payload)
        .expect("Response should decode");
    
    assert!(tree_response.status.is_some());
    assert_eq!(tree_response.status.unwrap().code, Code::Ok as i32);
    assert!(!tree_response.tree.is_empty());
}