// Catalog-specific test helpers
pub mod once_cell_app;
pub mod spawn_app;
pub mod test_service;
pub mod test_setup;

use catalog_messages::{
    CategoryResponse, Code, CreateCategoryRequest, ProductCreateRequest, ProductCreateResponse,
    ProductDeleteRequest, ProductDeleteResponse, ProductGetBySlugRequest, ProductGetBySlugResponse,
    ProductGetRequest, ProductGetResponse, ProductSearchRequest, ProductSearchResponse,
};
use prost::Message;
use rust_common::test_helpers::*;

// Import common module for generated proto code
mod common {
    pub use shared_proto::common::*;
}

// Include the generated protobuf messages
pub mod catalog_messages {
    include!(concat!(env!("OUT_DIR"), "/catalog_messages.rs"));

    // Re-export common types for backward compatibility
    pub use super::common::Code;
}

/// Helper to create a product and return its ID
pub async fn create_test_product(
    app: &TestApp,
    builder: fixtures::product::ProductBuilder,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let seo_title = format!("{} SEO", builder.name);
    let request = ProductCreateRequest {
        name: builder.name,
        product_ref: builder.product_ref,
        slug: builder.slug,
        brand: builder.brand,
        long_description: builder.long_description,
        product_type: builder.product_type,
        display_on_site: builder.display_on_site,
        defining_attributes: builder.defining_attributes,
        descriptive_attributes: builder.descriptive_attributes,
        seo_title: Some(seo_title),
        seo_description: Some("Test SEO description".to_string()),
        seo_keywords: Some("test,product".to_string()),
        tax_code: Some("txcd_99999999".to_string()),
        related_products: vec![],
        reviews: None,
        hierarchical_categories: None,
        list_categories: vec!["Test".to_string()],
        default_variant: None,
        variants: vec![],
    };

    let response = app
        .request("catalog.create_product", request.encode_to_vec())
        .await?;

    let create_response = ProductCreateResponse::decode(&*response.payload)?;

    if let Some(status) = &create_response.status {
        if status.code == Code::Ok as i32 {
            if let Some(product) = &create_response.product {
                return Ok(product.id.as_ref().unwrap_or(&String::new()).clone());
            }
        }
    }

    Err("Failed to create product".into())
}

/// Helper to get a product by ID
pub async fn get_product(
    app: &TestApp,
    id: &str,
) -> Result<ProductGetResponse, Box<dyn std::error::Error + Send + Sync>> {
    let request = ProductGetRequest { id: id.to_string() };

    let response = app
        .request("catalog.get_product", request.encode_to_vec())
        .await?;

    Ok(ProductGetResponse::decode(&*response.payload)?)
}

/// Helper to get a product by slug
pub async fn get_product_by_slug(
    app: &TestApp,
    slug: &str,
) -> Result<ProductGetBySlugResponse, Box<dyn std::error::Error + Send + Sync>> {
    let request = ProductGetBySlugRequest {
        slug: slug.to_string(),
    };

    let response = app
        .request("catalog.get_product_by_slug", request.encode_to_vec())
        .await?;

    Ok(ProductGetBySlugResponse::decode(&*response.payload)?)
}

/// Helper to delete a product
pub async fn delete_product(
    app: &TestApp,
    id: &str,
) -> Result<ProductDeleteResponse, Box<dyn std::error::Error + Send + Sync>> {
    let request = ProductDeleteRequest { id: id.to_string() };

    let response = app
        .request("catalog.delete_product", request.encode_to_vec())
        .await?;

    Ok(ProductDeleteResponse::decode(&*response.payload)?)
}

/// Helper to search products
pub async fn search_products(
    app: &TestApp,
    query: Option<String>,
    category: Option<String>,
    brand: Option<String>,
) -> Result<ProductSearchResponse, Box<dyn std::error::Error + Send + Sync>> {
    let request = ProductSearchRequest {
        query,
        categories: category.map(|c| vec![c]).unwrap_or_default(),
        brand,
        limit: Some(10),
        offset: None,
    };

    let response = app
        .request("catalog.search_products", request.encode_to_vec())
        .await?;

    Ok(ProductSearchResponse::decode(&*response.payload)?)
}

/// Helper to create a category and return its ID
pub async fn create_test_category(
    app: &TestApp,
    builder: fixtures::category::CategoryBuilder,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let request = CreateCategoryRequest {
        name: builder.name,
        slug: builder.slug,
        short_description: builder.short_description,
        full_description: None,
        parent_id: builder.parent_id,
        display_order: builder.display_order,
        seo: None,
        is_active: Some(true),
        parent_slug: None,
    };

    let response = app
        .request("catalog.create_category", request.encode_to_vec())
        .await?;

    let category_response = CategoryResponse::decode(&*response.payload)?;

    // CategoryResponse is the actual category object, not a wrapper
    Ok(category_response.id)
}

/// Assertion helpers specific to catalog
pub mod assertions {
    use super::*;

    pub fn assert_product_created(response: &ProductCreateResponse) {
        assert!(
            response.product.is_some(),
            "Product should be present in response"
        );
        assert!(response.status.is_some(), "Status should be present");

        let status = response.status.as_ref().unwrap();
        assert_eq!(status.code, Code::Ok as i32, "Status should be OK");

        let product = response.product.as_ref().unwrap();
        assert!(product.id.is_some(), "Product should have an ID");
        assert!(
            !product.id.as_ref().unwrap().is_empty(),
            "Product ID should not be empty"
        );
    }

    pub fn assert_product_not_found(response: &ProductGetResponse) {
        assert!(response.product.is_none(), "Product should not be present");
        assert!(response.status.is_some(), "Status should be present");

        let status = response.status.as_ref().unwrap();
        assert_eq!(
            status.code,
            Code::NotFound as i32,
            "Status should be NotFound"
        );
    }

    pub fn assert_invalid_request(status: &Option<common::Status>) {
        assert!(status.is_some(), "Status should be present");
        let status = status.as_ref().unwrap();
        assert_eq!(
            status.code,
            Code::InvalidArgument as i32,
            "Status should be InvalidArgument"
        );
    }
}
