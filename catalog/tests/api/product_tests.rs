use crate::helpers::catalog_messages::*;
use crate::helpers::{self, *};
use prost::Message;
use rust_common::test_helpers::{fixtures};
use std::collections::HashMap;

// ============================================================================
// PRODUCT CREATE TESTS
// ============================================================================

#[tokio::test]
async fn test_product_create_with_all_fields() {
    let app = helpers::spawn_app::spawn_app().await;
    let builder = fixtures::product::ProductBuilder::default();

    let request = ProductCreateRequest {
        name: builder.name.clone(),
        product_ref: builder.product_ref.clone(),
        slug: builder.slug.clone(),
        brand: builder.brand.clone(),
        long_description: builder.long_description.clone(),
        product_type: builder.product_type.clone(),
        display_on_site: builder.display_on_site,
        defining_attributes: builder.defining_attributes.clone(),
        descriptive_attributes: builder.descriptive_attributes.clone(),
        seo_title: Some("SEO Title".to_string()),
        seo_description: Some("SEO Description".to_string()),
        seo_keywords: Some("test,product,seo".to_string()),
        tax_code: Some("txcd_99999999".to_string()),
        related_products: vec!["prod-123".to_string()],
        reviews: None,
        hierarchical_categories: None,
        list_categories: vec!["Electronics".to_string(), "Computers".to_string()],
        default_variant: None,
        variants: vec![],
    };

    let response = app
        .request("catalog.create_product", request.encode_to_vec())
        .await
        .expect("Request should succeed");

    let create_response =
        ProductCreateResponse::decode(&*response.payload).expect("Response should decode");

    helpers::assertions::assert_product_created(&create_response);

    let product = create_response.product.unwrap();
    assert_eq!(product.name, builder.name);
    assert_eq!(product.product_ref, builder.product_ref);
    assert_eq!(product.slug, builder.slug);
}

#[tokio::test]
async fn test_product_create_with_minimal_fields() {
    let app = helpers::spawn_app::spawn_app().await;
    let builder = fixtures::product::ProductBuilder::minimal();

    let request = ProductCreateRequest {
        name: builder.name.clone(),
        product_ref: builder.product_ref.clone(),
        slug: None,
        brand: None,
        long_description: None,
        product_type: None,
        display_on_site: false,
        defining_attributes: HashMap::new(),
        descriptive_attributes: HashMap::new(),
        seo_title: None,
        seo_description: None,
        seo_keywords: None,
        tax_code: None,
        related_products: vec![],
        reviews: None,
        hierarchical_categories: None,
        list_categories: vec![],
        default_variant: None,
        variants: vec![],
    };

    let response = app
        .request("catalog.create_product", request.encode_to_vec())
        .await
        .expect("Request should succeed");

    let create_response =
        ProductCreateResponse::decode(&*response.payload).expect("Response should decode");

    helpers::assertions::assert_product_created(&create_response);
}

#[tokio::test]
async fn test_product_create_fails_with_missing_name() {
    let app = helpers::spawn_app::spawn_app().await;

    let request = ProductCreateRequest {
        name: "".to_string(), // Empty name should fail
        product_ref: fixtures::unique_product_ref(),
        slug: None,
        brand: None,
        long_description: None,
        product_type: None,
        display_on_site: false,
        defining_attributes: HashMap::new(),
        descriptive_attributes: HashMap::new(),
        seo_title: None,
        seo_description: None,
        seo_keywords: None,
        tax_code: None,
        related_products: vec![],
        reviews: None,
        hierarchical_categories: None,
        list_categories: vec![],
        default_variant: None,
        variants: vec![],
    };

    let response = app
        .request("catalog.create_product", request.encode_to_vec())
        .await
        .expect("Request should succeed");

    let create_response =
        ProductCreateResponse::decode(&*response.payload).expect("Response should decode");

    helpers::assertions::assert_invalid_request(&create_response.status);
}

#[tokio::test]
async fn test_product_create_fails_with_missing_product_ref() {
    let app = helpers::spawn_app::spawn_app().await;

    let request = ProductCreateRequest {
        name: "Test Product".to_string(),
        product_ref: "".to_string(), // Empty product_ref should fail
        slug: None,
        brand: None,
        long_description: None,
        product_type: None,
        display_on_site: false,
        defining_attributes: HashMap::new(),
        descriptive_attributes: HashMap::new(),
        seo_title: None,
        seo_description: None,
        seo_keywords: None,
        tax_code: None,
        related_products: vec![],
        reviews: None,
        hierarchical_categories: None,
        list_categories: vec![],
        default_variant: None,
        variants: vec![],
    };

    let response = app
        .request("catalog.create_product", request.encode_to_vec())
        .await
        .expect("Request should succeed");

    let create_response =
        ProductCreateResponse::decode(&*response.payload).expect("Response should decode");

    helpers::assertions::assert_invalid_request(&create_response.status);
}

#[tokio::test]
async fn test_product_create_with_sql_injection_attempt() {
    let app = helpers::spawn_app::spawn_app().await;

    for sql_injection in fixtures::invalid::sql_injection_strings() {
        let request = ProductCreateRequest {
            name: sql_injection.clone(),
            product_ref: fixtures::unique_product_ref(),
            slug: None,
            brand: None,
            long_description: None,
            product_type: None,
            display_on_site: false,
            defining_attributes: HashMap::new(),
            descriptive_attributes: HashMap::new(),
            seo_title: None,
            seo_description: None,
            seo_keywords: None,
            tax_code: None,
            related_products: vec![],
            reviews: None,
            hierarchical_categories: None,
            list_categories: vec![],
            default_variant: None,
            variants: vec![],
        };

        let response = app
            .request("catalog.create_product", request.encode_to_vec())
            .await
            .expect("Request should succeed");

        let create_response =
            ProductCreateResponse::decode(&*response.payload).expect("Response should decode");

        // Should either reject as invalid or safely handle the input
        if create_response.product.is_some() {
            let product = create_response.product.unwrap();
            // Verify the SQL injection string was treated as plain text
            assert_eq!(product.name, sql_injection);
        }
    }
}

// ============================================================================
// PRODUCT GET TESTS
// ============================================================================

#[tokio::test]
async fn test_product_get_existing_product() {
    let app = helpers::spawn_app::spawn_app().await;

    // Create a product first
    let builder = fixtures::product::ProductBuilder::default();
    let expected_name = builder.name.clone();
    let product_id = create_test_product(&app, builder)
        .await
        .expect("Should create product");

    // Get the product
    let response = get_product(&app, &product_id)
        .await
        .expect("Should get product");

    assert!(response.product.is_some());
    assert!(response.status.is_some());
    assert_eq!(response.status.unwrap().code, Code::Ok as i32);

    let product = response.product.unwrap();
    assert_eq!(product.id, Some(product_id));
    assert_eq!(product.name, expected_name);
}

#[tokio::test]
async fn test_product_get_non_existent_product() {
    let app = helpers::spawn_app::spawn_app().await;

    let response = get_product(&app, "non-existent-id")
        .await
        .expect("Should get response");

    helpers::assertions::assert_product_not_found(&response);
}

#[tokio::test]
async fn test_product_get_with_empty_id() {
    let app = helpers::spawn_app::spawn_app().await;

    let response = get_product(&app, "").await.expect("Should get response");

    helpers::assertions::assert_invalid_request(&response.status);
}

#[tokio::test]
async fn test_product_get_with_invalid_id_format() {
    let app = helpers::spawn_app::spawn_app().await;

    // Test various invalid ID formats
    let invalid_ids = vec![
        "'; DROP TABLE products; --",
        "../../../etc/passwd",
        "<script>alert('xss')</script>",
        "\\x00\\x01\\x02",
    ];

    for invalid_id in invalid_ids {
        let response = get_product(&app, invalid_id)
            .await
            .expect("Should get response");

        // Should either return not found or invalid argument
        assert!(response.product.is_none());
        assert!(response.status.is_some());
        let status = response.status.unwrap();
        assert!(
            status.code == Code::NotFound as i32 || status.code == Code::InvalidArgument as i32
        );
    }
}

// ============================================================================
// PRODUCT GET BY SLUG TESTS
// ============================================================================

#[tokio::test]
async fn test_product_get_by_slug_existing() {
    let app = helpers::spawn_app::spawn_app().await;

    // Create a product with a specific slug
    let slug = fixtures::valid_slug();
    let mut builder = fixtures::product::ProductBuilder::default();
    builder.slug = Some(slug.clone());

    let product_id = create_test_product(&app, builder.clone())
        .await
        .expect("Should create product");

    // Get by slug
    let response = get_product_by_slug(&app, &slug)
        .await
        .expect("Should get product");

    assert!(response.product.is_some());
    assert!(response.status.is_some());
    assert_eq!(response.status.unwrap().code, Code::Ok as i32);

    let product = response.product.unwrap();
    assert_eq!(product.id, Some(product_id));
    assert_eq!(product.slug, Some(slug));
}

#[tokio::test]
async fn test_product_get_by_slug_non_existent() {
    let app = helpers::spawn_app::spawn_app().await;

    let response = get_product_by_slug(&app, "non-existent-slug")
        .await
        .expect("Should get response");

    assert!(response.product.is_none());
    assert!(response.status.is_some());
    assert_eq!(response.status.unwrap().code, Code::NotFound as i32);
}

#[tokio::test]
async fn test_product_get_by_slug_with_special_characters() {
    let app = helpers::spawn_app::spawn_app().await;

    // Test slugs with special characters
    let special_slugs = vec![
        "product-with-dash",
        "product_with_underscore",
        "product.with.dots",
        "product123",
    ];

    for slug in special_slugs {
        let mut builder = fixtures::product::ProductBuilder::default();
        builder.slug = Some(slug.to_string());

        let product_id = create_test_product(&app, builder)
            .await
            .expect("Should create product");

        let response = get_product_by_slug(&app, slug)
            .await
            .expect("Should get product");

        assert!(response.product.is_some());
        let product = response.product.unwrap();
        assert_eq!(product.id, Some(product_id));
        assert_eq!(product.slug, Some(slug.to_string()));
    }
}

#[tokio::test]
async fn test_product_get_by_slug_sql_injection() {
    let app = helpers::spawn_app::spawn_app().await;

    for sql_injection in fixtures::invalid::sql_injection_strings() {
        let response = get_product_by_slug(&app, &sql_injection)
            .await
            .expect("Should get response");

        // Should safely handle SQL injection attempts
        assert!(response.product.is_none());
        assert!(response.status.is_some());
    }
}

// ============================================================================
// PRODUCT DELETE TESTS
// ============================================================================

#[tokio::test]
async fn test_product_delete_existing() {
    let app = helpers::spawn_app::spawn_app().await;

    // Create a product
    let builder = fixtures::product::ProductBuilder::default();
    let product_id = create_test_product(&app, builder)
        .await
        .expect("Should create product");

    // Delete the product
    let response = delete_product(&app, &product_id)
        .await
        .expect("Should delete product");

    assert!(response.status.is_some());
    assert_eq!(response.status.unwrap().code, Code::Ok as i32);

    // Verify it's deleted
    let get_response = get_product(&app, &product_id)
        .await
        .expect("Should get response");

    assertions::assert_product_not_found(&get_response);
}

#[tokio::test]
async fn test_product_delete_non_existent() {
    let app = helpers::spawn_app::spawn_app().await;

    let response = delete_product(&app, "non-existent-id")
        .await
        .expect("Should get response");

    // Deleting non-existent should still return OK (idempotent)
    assert!(response.status.is_some());
    let status = response.status.unwrap();
    assert!(status.code == Code::Ok as i32 || status.code == Code::NotFound as i32);
}

#[tokio::test]
async fn test_product_delete_idempotent() {
    let app = helpers::spawn_app::spawn_app().await;

    // Create and delete a product
    let builder = fixtures::product::ProductBuilder::default();
    let product_id = create_test_product(&app, builder)
        .await
        .expect("Should create product");

    // Delete twice
    let response1 = delete_product(&app, &product_id)
        .await
        .expect("Should delete product");
    assert_eq!(response1.status.unwrap().code, Code::Ok as i32);

    let response2 = delete_product(&app, &product_id)
        .await
        .expect("Should delete product again");

    // Second delete should also succeed (idempotent)
    assert!(response2.status.is_some());
    let status = response2.status.unwrap();
    assert!(status.code == Code::Ok as i32 || status.code == Code::NotFound as i32);
}

// ============================================================================
// PRODUCT SEARCH TESTS
// ============================================================================

#[tokio::test]
async fn test_product_search_by_name() {
    let app = helpers::spawn_app::spawn_app().await;

    // Create products with specific names
    let search_term = "SpecialSearchProduct";
    let mut builder1 = fixtures::product::ProductBuilder::default();
    builder1.name = format!("{search_term} One");

    let mut builder2 = fixtures::product::ProductBuilder::default();
    builder2.name = format!("{search_term} Two");

    let mut builder3 = fixtures::product::ProductBuilder::default();
    builder3.name = "Different Product".to_string();

    create_test_product(&app, builder1)
        .await
        .expect("Should create product 1");
    create_test_product(&app, builder2)
        .await
        .expect("Should create product 2");
    create_test_product(&app, builder3)
        .await
        .expect("Should create product 3");

    // Search for products
    let response = search_products(&app, Some(search_term.to_string()), None, None)
        .await
        .expect("Should search products");

    assert!(response.products.len() >= 2);
    assert!(response.status.is_some());
    assert_eq!(response.status.unwrap().code, Code::Ok as i32);

    // Verify search results contain the search term
    for product in &response.products {
        assert!(product.name.contains(search_term));
    }
}

#[tokio::test]
async fn test_product_search_empty_query() {
    let app = helpers::spawn_app::spawn_app().await;

    // Create some products
    for _ in 0..3 {
        let builder = fixtures::product::ProductBuilder::default();
        create_test_product(&app, builder)
            .await
            .expect("Should create product");
    }

    // Search with empty query should return all products
    let response = search_products(&app, None, None, None)
        .await
        .expect("Should search products");

    assert!(response.status.is_some());
    assert_eq!(response.status.unwrap().code, Code::Ok as i32);
    assert!(!response.products.is_empty());
}

#[tokio::test]
async fn test_product_search_sql_injection() {
    let app = helpers::spawn_app::spawn_app().await;

    for sql_injection in fixtures::invalid::sql_injection_strings() {
        let response = search_products(&app, Some(sql_injection), None, None)
            .await
            .expect("Should get response");

        // Should safely handle SQL injection attempts
        assert!(response.status.is_some());
        assert_eq!(response.status.unwrap().code, Code::Ok as i32);
        // Results should be empty or safely filtered
    }
}

#[tokio::test]
async fn test_product_search_with_xss_attempt() {
    let app = helpers::spawn_app::spawn_app().await;

    for xss_string in fixtures::invalid::xss_strings() {
        let response = search_products(&app, Some(xss_string.clone()), None, None)
            .await
            .expect("Should get response");

        // Should safely handle XSS attempts
        assert!(response.status.is_some());
        assert_eq!(response.status.unwrap().code, Code::Ok as i32);

        // If any results are returned, verify XSS is escaped/sanitized
        for product in &response.products {
            assert!(!product.name.contains("<script"));
            assert!(!product.name.contains("javascript:"));
        }
    }
}

// ============================================================================
// PRODUCT EXPORT TESTS
// ============================================================================

// #[tokio::test]
// async fn test_product_export_empty_catalog() {
//     use crate::helpers::once_cell_app::{cleanup_products, get_test_app};

//     // Get the shared test app
//     let (app, db_name, client) = get_test_app().await;
//     eprintln!("Test: export_empty_catalog - using database: {db_name}");

//     // Clean up any existing products to ensure empty catalog
//     cleanup_products(&client, &db_name).await;

//     let request = ProductExportRequest {
//         batch_size: Some(10),
//         offset: None,
//     };

//     let response = app
//         .request("catalog.export_products", request.encode_to_vec())
//         .await
//         .expect("Request should succeed");

//     let export_response =
//         ProductExportResponse::decode(&*response.payload).expect("Response should decode");

//     assert!(export_response.status.is_some());
//     assert_eq!(export_response.status.unwrap().code, Code::Ok as i32);

//     // Debug: print the number of products if not empty
//     if !export_response.products.is_empty() {
//         eprintln!(
//             "Expected empty catalog but found {} products",
//             export_response.products.len()
//         );
//         for product in &export_response.products {
//             eprintln!("  - Product: {} ({})", product.name, product.product_ref);
//         }
//     }

//     assert!(
//         export_response.products.is_empty(),
//         "Catalog should be empty but contains {} products",
//         export_response.products.len()
//     );
// }

#[tokio::test]
async fn test_product_export_with_products() {
    let app = helpers::spawn_app::spawn_app().await;

    // Create some products to export
    let mut product_ids = vec![];
    for i in 0..3 {
        let mut builder = fixtures::product::ProductBuilder::default();
        builder.name = format!("Export Test Product {i}");
        let id = create_test_product(&app, builder)
            .await
            .expect("Should create product");
        product_ids.push(id);
    }

    let request = ProductExportRequest {
        batch_size: Some(10),
        offset: None,
    };

    let response = app
        .request("catalog.export_products", request.encode_to_vec())
        .await
        .expect("Request should succeed");

    let export_response =
        ProductExportResponse::decode(&*response.payload).expect("Response should decode");

    assert!(export_response.status.is_some());
    assert_eq!(export_response.status.unwrap().code, Code::Ok as i32);
    assert!(export_response.products.len() >= 3);

    // Verify exported products have expected fields
    for product in &export_response.products {
        assert!(product.id.is_some());
        assert!(!product.name.is_empty());
        assert!(!product.product_ref.is_empty());
    }
}

#[tokio::test]
async fn test_product_export_with_pagination() {
    let app = helpers::spawn_app::spawn_app().await;

    // Create multiple products
    for i in 0..5 {
        let mut builder = fixtures::product::ProductBuilder::default();
        builder.name = format!("Paginated Product {i}");
        create_test_product(&app, builder)
            .await
            .expect("Should create product");
    }

    // Export with small batch size
    let request = ProductExportRequest {
        batch_size: Some(2),
        offset: Some(0),
    };

    let response = app
        .request("catalog.export_products", request.encode_to_vec())
        .await
        .expect("Request should succeed");

    let export_response =
        ProductExportResponse::decode(&*response.payload).expect("Response should decode");

    assert!(export_response.status.is_some());
    assert_eq!(export_response.status.unwrap().code, Code::Ok as i32);
    assert!(export_response.products.len() <= 2);

    // Export next batch
    let request = ProductExportRequest {
        batch_size: Some(2),
        offset: Some(2),
    };

    let response = app
        .request("catalog.export_products", request.encode_to_vec())
        .await
        .expect("Request should succeed");

    let export_response =
        ProductExportResponse::decode(&*response.payload).expect("Response should decode");

    assert!(export_response.status.is_some());
    assert_eq!(export_response.status.unwrap().code, Code::Ok as i32);
}

// Note: Import is handled via the client which reads JSON files and creates products
// individually, not through a bulk import message. We test the create functionality
// extensively above which covers the import use case.
