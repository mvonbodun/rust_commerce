#[cfg(test)]
mod product_slug_integration_tests {
    
    use prost::Message;
    use std::collections::HashMap;
    
    use uuid::Uuid;

    // Include the generated protobuf messages
    pub mod catalog_messages {
        include!(concat!(env!("OUT_DIR"), "/catalog_messages.rs"));
    }

    use catalog_messages::{
        Code, ProductCreateRequest, ProductCreateResponse, ProductDeleteRequest,
        ProductDeleteResponse, ProductGetBySlugRequest, ProductGetBySlugResponse,
    };

    async fn setup_nats_client() -> async_nats::Client {
        let url =
            std::env::var("NATS_TEST_URL").unwrap_or_else(|_| "nats://127.0.0.1:4222".to_string());
        async_nats::connect(url)
            .await
            .expect("Failed to connect to NATS")
    }

    async fn create_test_product(
        client: &async_nats::Client,
        name: &str,
        slug: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let product_request = ProductCreateRequest {
            name: name.to_string(),
            long_description: Some(format!("Test description for {name}")),
            brand: Some("TestBrand".to_string()),
            slug: Some(slug.to_string()),
            product_ref: format!("TEST-{}", Uuid::new_v4().to_string()[0..8].to_uppercase()),
            product_type: Some("simple".to_string()),
            seo_title: Some(name.to_string()),
            seo_description: Some(format!("SEO description for {name}")),
            seo_keywords: Some("test, product, integration".to_string()),
            display_on_site: true,
            tax_code: Some("txcd_99999999".to_string()),
            related_products: vec![],
            reviews: None,
            hierarchical_categories: None,
            list_categories: vec!["Test Category".to_string()],
            defining_attributes: HashMap::new(),
            descriptive_attributes: HashMap::new(),
            default_variant: None,
            variants: vec![],
        };

        let request_bytes = product_request.encode_to_vec();
        let response = client
            .request("catalog.create_product", request_bytes.into())
            .await?;

        let create_response = ProductCreateResponse::decode(&*response.payload)?;

        if let Some(status) = &create_response.status {
            if status.code == Code::Ok as i32 {
                if let Some(product) = &create_response.product {
                    Ok(product.id.as_ref().unwrap_or(&String::new()).clone())
                } else {
                    Err("No product in create response".into())
                }
            } else {
                Err(format!(
                    "Failed to create product: {} ({})",
                    status.message, status.code
                )
                .into())
            }
        } else {
            Err("No status in create response".into())
        }
    }

    async fn cleanup_product(
        client: &async_nats::Client,
        product_id: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let delete_request = ProductDeleteRequest {
            id: product_id.to_string(),
        };

        let request_bytes = delete_request.encode_to_vec();
        let response = client
            .request("catalog.delete_product", request_bytes.into())
            .await?;

        let delete_response = ProductDeleteResponse::decode(&*response.payload)?;

        if let Some(status) = &delete_response.status {
            if status.code == Code::Ok as i32 {
                Ok(())
            } else {
                Err(format!(
                    "Failed to delete product: {} ({})",
                    status.message, status.code
                )
                .into())
            }
        } else {
            Err("No status in delete response".into())
        }
    }

    #[tokio::test]
    async fn test_get_product_by_slug_success() {
        let client = setup_nats_client().await;

        // Create a test product
        let uuid_short = &Uuid::new_v4().to_string()[0..8];
    let test_slug = format!("test-product-{uuid_short}");
        let product_id = create_test_product(&client, "Test Product", &test_slug)
            .await
            .expect("Failed to create test product");

        // Test getting the product by slug
        let get_request = ProductGetBySlugRequest {
            slug: test_slug.clone(),
        };

        let request_bytes = get_request.encode_to_vec();
        let response = client
            .request("catalog.get_product_by_slug", request_bytes.into())
            .await
            .expect("Failed to send get_product_by_slug request");

        let get_response = ProductGetBySlugResponse::decode(&*response.payload)
            .expect("Failed to decode response");

        // Verify the response
        assert!(get_response.status.is_some());
        let status = get_response.status.unwrap();
        assert_eq!(
            status.code,
            Code::Ok as i32,
            "Expected OK status, got: {}",
            status.message
        );

        assert!(get_response.product.is_some());
        let product = get_response.product.unwrap();
        assert_eq!(product.name, "Test Product");
        assert_eq!(product.slug, Some(test_slug));
        assert_eq!(product.brand, Some("TestBrand".to_string()));

        // Cleanup
        cleanup_product(&client, &product_id)
            .await
            .expect("Failed to cleanup test product");
    }

    #[tokio::test]
    async fn test_get_product_by_slug_not_found() {
        let client = setup_nats_client().await;

        // Test getting a product with a non-existent slug
        let uuid_short = &Uuid::new_v4().to_string()[0..8];
    let non_existent_slug = format!("non-existent-{uuid_short}");
        let get_request = ProductGetBySlugRequest {
            slug: non_existent_slug,
        };

        let request_bytes = get_request.encode_to_vec();
        let response = client
            .request("catalog.get_product_by_slug", request_bytes.into())
            .await
            .expect("Failed to send get_product_by_slug request");

        let get_response = ProductGetBySlugResponse::decode(&*response.payload)
            .expect("Failed to decode response");

        // Verify the response indicates not found
        assert!(get_response.status.is_some());
        let status = get_response.status.unwrap();
        assert_eq!(
            status.code,
            Code::NotFound as i32,
            "Expected NotFound status, got: {}",
            status.message
        );
        assert!(
            get_response.product.is_none(),
            "Expected no product in response"
        );
    }

    #[tokio::test]
    async fn test_get_product_by_slug_empty_slug() {
        let client = setup_nats_client().await;

        // Test getting a product with an empty slug
        let get_request = ProductGetBySlugRequest {
            slug: "".to_string(),
        };

        let request_bytes = get_request.encode_to_vec();
        let response = client
            .request("catalog.get_product_by_slug", request_bytes.into())
            .await
            .expect("Failed to send get_product_by_slug request");

        let get_response = ProductGetBySlugResponse::decode(&*response.payload)
            .expect("Failed to decode response");

        // Verify the response indicates invalid request
        assert!(get_response.status.is_some());
        let status = get_response.status.unwrap();
        assert_ne!(
            status.code,
            Code::Ok as i32,
            "Expected error status for empty slug"
        );
        assert!(
            get_response.product.is_none(),
            "Expected no product in response"
        );
    }

    #[tokio::test]
    async fn test_get_product_by_slug_case_sensitivity() {
        let client = setup_nats_client().await;

        // Create a test product with a specific slug
        let uuid_short = &Uuid::new_v4().to_string()[0..8];
    let test_slug = format!("test-case-product-{uuid_short}");
        let product_id = create_test_product(&client, "Test Case Product", &test_slug)
            .await
            .expect("Failed to create test product");

        // Test getting the product with different case
        let uppercase_slug = test_slug.to_uppercase();
        let get_request = ProductGetBySlugRequest {
            slug: uppercase_slug,
        };

        let request_bytes = get_request.encode_to_vec();
        let response = client
            .request("catalog.get_product_by_slug", request_bytes.into())
            .await
            .expect("Failed to send get_product_by_slug request");

        let get_response = ProductGetBySlugResponse::decode(&*response.payload)
            .expect("Failed to decode response");

        // Verify the response - slugs should be case sensitive (not found)
        assert!(get_response.status.is_some());
        let status = get_response.status.unwrap();
        assert_eq!(
            status.code,
            Code::NotFound as i32,
            "Expected NotFound status for case mismatch"
        );
        assert!(
            get_response.product.is_none(),
            "Expected no product in response for case mismatch"
        );

        // Test with exact case - should work
        let exact_case_request = ProductGetBySlugRequest {
            slug: test_slug.clone(),
        };

        let request_bytes = exact_case_request.encode_to_vec();
        let response = client
            .request("catalog.get_product_by_slug", request_bytes.into())
            .await
            .expect("Failed to send get_product_by_slug request");

        let get_response = ProductGetBySlugResponse::decode(&*response.payload)
            .expect("Failed to decode response");

        // Verify exact case works
        assert!(get_response.status.is_some());
        let status = get_response.status.unwrap();
        assert_eq!(
            status.code,
            Code::Ok as i32,
            "Expected OK status for exact case match"
        );
        assert!(
            get_response.product.is_some(),
            "Expected product in response for exact case match"
        );

        // Cleanup
        cleanup_product(&client, &product_id)
            .await
            .expect("Failed to cleanup test product");
    }

    #[tokio::test]
    async fn test_get_product_by_slug_special_characters() {
        let client = setup_nats_client().await;

        // Create a test product with special characters in slug
        let uuid_short = &Uuid::new_v4().to_string()[0..8];
    let test_slug = format!("test-special-chars-{uuid_short}-product");
        let product_id = create_test_product(&client, "Test Special Chars Product", &test_slug)
            .await
            .expect("Failed to create test product");

        // Test getting the product by slug with special characters
        let get_request = ProductGetBySlugRequest {
            slug: test_slug.clone(),
        };

        let request_bytes = get_request.encode_to_vec();
        let response = client
            .request("catalog.get_product_by_slug", request_bytes.into())
            .await
            .expect("Failed to send get_product_by_slug request");

        let get_response = ProductGetBySlugResponse::decode(&*response.payload)
            .expect("Failed to decode response");

        // Verify the response
        assert!(get_response.status.is_some());
        let status = get_response.status.unwrap();
        assert_eq!(
            status.code,
            Code::Ok as i32,
            "Expected OK status, got: {}",
            status.message
        );

        assert!(get_response.product.is_some());
        let product = get_response.product.unwrap();
        assert_eq!(product.slug, Some(test_slug));

        // Cleanup
        cleanup_product(&client, &product_id)
            .await
            .expect("Failed to cleanup test product");
    }

    #[tokio::test]
    async fn test_get_product_by_slug_performance() {
        let client = setup_nats_client().await;

        // Create a test product
        let uuid_short = &Uuid::new_v4().to_string()[0..8];
    let test_slug = format!("perf-test-{uuid_short}");
        let product_id = create_test_product(&client, "Performance Test Product", &test_slug)
            .await
            .expect("Failed to create test product");

        // Measure response time for multiple requests
        let mut response_times = Vec::new();
        let iterations = 10;

        for _ in 0..iterations {
            let start = std::time::Instant::now();

            let get_request = ProductGetBySlugRequest {
                slug: test_slug.clone(),
            };

            let request_bytes = get_request.encode_to_vec();
            let response = client
                .request("catalog.get_product_by_slug", request_bytes.into())
                .await
                .expect("Failed to send get_product_by_slug request");

            let get_response = ProductGetBySlugResponse::decode(&*response.payload)
                .expect("Failed to decode response");

            let duration = start.elapsed();
            response_times.push(duration.as_millis());

            // Verify each response is successful
            assert!(get_response.status.is_some());
            let status = get_response.status.unwrap();
            assert_eq!(status.code, Code::Ok as i32);
        }

        // Calculate average response time
        let avg_response_time = response_times.iter().sum::<u128>() / iterations as u128;
        println!(
            "Average response time for get_product_by_slug: {avg_response_time}ms",
        );

        // Assert reasonable performance (under 100ms for local testing)
        assert!(
            avg_response_time < 100,
            "Average response time {avg_response_time}ms exceeds 100ms threshold",
        );

        // Cleanup
        cleanup_product(&client, &product_id)
            .await
            .expect("Failed to cleanup test product");
    }
}
