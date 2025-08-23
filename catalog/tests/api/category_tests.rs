use crate::helpers::*;
use crate::helpers::{self, catalog_messages::*};
use prost::Message;
use rust_common::test_helpers::fixtures;
use shared_proto::common::Code;

// ============================================================================
// CATEGORY CREATE TESTS
// ============================================================================

#[tokio::test]
async fn test_category_create_root_category() {
    let app = helpers::spawn_app::spawn_app().await;
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
        .request(
            crate::helpers::nats_config::category::subjects::CREATE_CATEGORY,
            request.encode_to_vec(),
        )
        .await
        .expect("Request should succeed");

    let create_response =
        CreateCategoryResponse::decode(&*response.payload).expect("Response should decode");

    // Check status
    assert!(create_response.status.is_some());
    let status = create_response.status.as_ref().unwrap();
    assert_eq!(status.code, Code::Ok as i32);

    // Check category
    assert!(create_response.category.is_some());
    let category = create_response.category.as_ref().unwrap();
    assert!(!category.id.is_empty());
    assert_eq!(category.name, builder.name);
    assert_eq!(category.slug, builder.slug);
}

#[tokio::test]
async fn test_category_create_child_category() {
    let app = helpers::spawn_app::spawn_app().await;

    // Create parent category first
    let parent_builder = fixtures::category::CategoryBuilder::root();
    let parent_id = create_test_category(&app, parent_builder)
        .await
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
        .request(
            crate::helpers::nats_config::category::subjects::CREATE_CATEGORY,
            request.encode_to_vec(),
        )
        .await
        .expect("Request should succeed");

    let create_response =
        CreateCategoryResponse::decode(&*response.payload).expect("Response should decode");

    // Check status
    assert!(create_response.status.is_some());
    let status = create_response.status.as_ref().unwrap();
    assert_eq!(status.code, Code::Ok as i32);

    // Check category
    assert!(create_response.category.is_some());
    let category = create_response.category.as_ref().unwrap();
    assert_eq!(category.parent_id, Some(parent_id));
}

// ============================================================================
// CATEGORY GET TESTS
// ============================================================================

#[tokio::test]
async fn test_category_get_existing() {
    let app = helpers::spawn_app::spawn_app().await;

    // Create a category
    let builder = fixtures::category::CategoryBuilder::default();
    let category_id = create_test_category(&app, builder.clone())
        .await
        .expect("Should create category");

    let request = GetCategoryRequest {
        id: category_id.clone(),
    };

    let response = app
        .request(
            crate::helpers::nats_config::category::subjects::GET_CATEGORY,
            request.encode_to_vec(),
        )
        .await
        .expect("Request should succeed");

    let get_response =
        GetCategoryResponse::decode(&*response.payload).expect("Response should decode");

    assert!(get_response.status.is_some());
    assert_eq!(get_response.status.unwrap().code, Code::Ok as i32);
    assert!(get_response.category.is_some());

    let category = get_response.category.unwrap();
    assert_eq!(category.id, category_id);
    assert_eq!(category.name, builder.name);
}

#[tokio::test]
async fn test_category_get_non_existent() {
    let app = helpers::spawn_app::spawn_app().await;

    let request = GetCategoryRequest {
        id: "non-existent-id".to_string(),
    };

    let response = app
        .request(
            crate::helpers::nats_config::category::subjects::GET_CATEGORY,
            request.encode_to_vec(),
        )
        .await;

    // Should return NotFound status for non-existent category
    let msg = response.expect("Should get response");
    let get_response = GetCategoryResponse::decode(&*msg.payload).expect("Should decode response");

    assert!(get_response.category.is_none());
    assert!(get_response.status.is_some());
    assert_eq!(get_response.status.unwrap().code, Code::NotFound as i32);
}

// ============================================================================
// CATEGORY GET BY SLUG TESTS
// ============================================================================

#[tokio::test]
async fn test_category_get_by_slug_existing() {
    let app = helpers::spawn_app::spawn_app().await;

    let slug = fixtures::valid_slug();
    let mut builder = fixtures::category::CategoryBuilder::default();
    builder.slug = slug.clone();

    let category_id = create_test_category(&app, builder)
        .await
        .expect("Should create category");

    let request = GetCategoryBySlugRequest { slug: slug.clone() };

    let response = app
        .request(
            crate::helpers::nats_config::category::subjects::GET_CATEGORY_BY_SLUG,
            request.encode_to_vec(),
        )
        .await
        .expect("Request should succeed");

    let get_response =
        GetCategoryBySlugResponse::decode(&*response.payload).expect("Response should decode");

    assert!(get_response.status.is_some());
    assert_eq!(get_response.status.unwrap().code, Code::Ok as i32);
    assert!(get_response.category.is_some());

    let category = get_response.category.unwrap();
    assert_eq!(category.id, category_id);
    assert_eq!(category.slug, slug);
}

// ============================================================================
// CATEGORY UPDATE TESTS
// ============================================================================

#[tokio::test]
async fn test_category_update_name_and_description() {
    let app = helpers::spawn_app::spawn_app().await;

    // Create a category
    let builder = fixtures::category::CategoryBuilder::default();
    let category_id = create_test_category(&app, builder)
        .await
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
        .request(
            crate::helpers::nats_config::category::subjects::UPDATE_CATEGORY,
            request.encode_to_vec(),
        )
        .await
        .expect("Request should succeed");

    let update_response =
        UpdateCategoryResponse::decode(&*response.payload).expect("Response should decode");

    // Check status
    assert!(update_response.status.is_some());
    let status = update_response.status.as_ref().unwrap();
    assert_eq!(status.code, Code::Ok as i32);

    // Check category
    assert!(update_response.category.is_some());
    let category = update_response.category.as_ref().unwrap();
    assert_eq!(category.name, "Updated Name");
    assert_eq!(category.short_description, "Updated short description");
}

// ============================================================================
// CATEGORY DELETE TESTS
// ============================================================================

#[tokio::test]
async fn test_category_delete_existing() {
    let app = helpers::spawn_app::spawn_app().await;

    // Create a category
    let builder = fixtures::category::CategoryBuilder::default();
    let category_id = create_test_category(&app, builder)
        .await
        .expect("Should create category");

    let request = DeleteCategoryRequest {
        id: category_id.clone(),
    };

    let response = app
        .request(
            crate::helpers::nats_config::category::subjects::DELETE_CATEGORY,
            request.encode_to_vec(),
        )
        .await
        .expect("Request should succeed");

    // Delete returns DeleteCategoryResponse
    let delete_response =
        DeleteCategoryResponse::decode(&*response.payload).expect("Response should decode");

    assert!(delete_response.status.is_some());
    let status = delete_response.status.as_ref().unwrap();
    assert_eq!(status.code, Code::Ok as i32);

    // Verify it's deleted by trying to get it
    let get_request = GetCategoryRequest { id: category_id };

    let get_response = app
        .request(
            crate::helpers::nats_config::category::subjects::GET_CATEGORY,
            get_request.encode_to_vec(),
        )
        .await
        .expect("Request should succeed");

    // Should return NotFound status for deleted category
    let response =
        GetCategoryResponse::decode(&*get_response.payload).expect("Response should decode");

    assert!(response.category.is_none());
    assert!(response.status.is_some());
    assert_eq!(response.status.unwrap().code, Code::NotFound as i32);
}

// ============================================================================
// CATEGORY TREE TESTS
// ============================================================================

#[tokio::test]
async fn test_category_tree_with_hierarchy() {
    let app = helpers::spawn_app::spawn_app().await;

    // Create a hierarchy: Root -> Child1, Child2
    let root = fixtures::category::CategoryBuilder::root();
    let root_id = create_test_category(&app, root)
        .await
        .expect("Should create root");

    let child1 = fixtures::category::CategoryBuilder::child_of(root_id.clone());
    create_test_category(&app, child1)
        .await
        .expect("Should create child1");

    let child2 = fixtures::category::CategoryBuilder::child_of(root_id.clone());
    create_test_category(&app, child2)
        .await
        .expect("Should create child2");

    // Get tree
    let request = CategoryTreeRequest {
        max_depth: Some(10),
        include_inactive: Some(false),
        rebuild_cache: None,
    };

    let response = app
        .request(
            crate::helpers::nats_config::category::subjects::GET_CATEGORY_TREE,
            request.encode_to_vec(),
        )
        .await
        .expect("Request should succeed");

    let tree_response =
        CategoryTreeResponse::decode(&*response.payload).expect("Response should decode");

    assert!(tree_response.status.is_some());
    assert_eq!(tree_response.status.unwrap().code, Code::Ok as i32);
    assert!(!tree_response.tree.is_empty());
}
