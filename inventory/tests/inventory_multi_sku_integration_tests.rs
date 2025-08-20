#[cfg(test)]
mod inventory_multi_sku_integration_tests {
    use prost::Message;
    use std::collections::HashMap;
    use uuid::Uuid;

    // Include the generated protobuf messages
    pub mod inventory_messages {
        include!(concat!(env!("OUT_DIR"), "/inventory_messages.rs"));
    }

    use inventory_messages::{
        Code, InventoryCreateRequest, InventoryCreateResponse, InventoryDeleteRequest,
        InventoryGetAllLocationsBySkuRequest, InventoryGetAllLocationsBySkuResponse,
    };

    async fn setup_nats_client() -> async_nats::Client {
        let url =
            std::env::var("NATS_TEST_URL").unwrap_or_else(|_| "nats://127.0.0.1:4222".to_string());
        async_nats::connect(url)
            .await
            .expect("Failed to connect to NATS")
    }

    async fn create_test_inventory_item(
        client: &async_nats::Client,
        sku: &str,
        location: &str,
        quantity: i32,
        reserved: i32,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let request = InventoryCreateRequest {
            sku: sku.to_string(),
            quantity,
            reserved_quantity: reserved,
            min_stock_level: 10,
            location: location.to_string(),
        };

        let request_bytes = request.encode_to_vec();
        let response = client
            .request("inventory.create_item", request_bytes.into())
            .await?;

        let create_response = InventoryCreateResponse::decode(&*response.payload)?;

        if let Some(status) = &create_response.status {
            if status.code == Code::Ok as i32 {
                if let Some(item) = &create_response.item {
                    Ok(item.id.as_ref().unwrap_or(&String::new()).clone())
                } else {
                    Err("No item in create response".into())
                }
            } else {
                Err(format!(
                    "Failed to create inventory item: {} ({})",
                    status.message, status.code
                )
                .into())
            }
        } else {
            Err("No status in create response".into())
        }
    }

    async fn cleanup_inventory_item(
        client: &async_nats::Client,
        sku: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let delete_request = InventoryDeleteRequest {
            sku: sku.to_string(),
        };

        let request_bytes = delete_request.encode_to_vec();
        let _response = client
            .request("inventory.delete_item", request_bytes.into())
            .await?;

        // Note: We don't check the response as items might not exist
        Ok(())
    }

    #[tokio::test]
    async fn test_get_all_locations_by_sku_success() {
        let client = setup_nats_client().await;

        // Create test inventory items for multiple SKUs across different locations
        let sku1 = format!(
            "TEST-SKU-{}",
            Uuid::new_v4().to_string()[0..8].to_uppercase()
        );
        let sku2 = format!(
            "TEST-SKU-{}",
            Uuid::new_v4().to_string()[0..8].to_uppercase()
        );

        // SKU1 in multiple locations
        create_test_inventory_item(&client, &sku1, "DC - Dallas, TX", 100, 20)
            .await
            .expect("Failed to create test inventory item");
        create_test_inventory_item(&client, &sku1, "STORE - Spring, TX", 50, 10)
            .await
            .expect("Failed to create test inventory item");
        create_test_inventory_item(&client, &sku1, "STORE - Austin, TX", 30, 5)
            .await
            .expect("Failed to create test inventory item");

        // SKU2 in single location
        create_test_inventory_item(&client, &sku2, "DC - Dallas, TX", 75, 15)
            .await
            .expect("Failed to create test inventory item");

        // Test getting inventory for both SKUs
        let request = InventoryGetAllLocationsBySkuRequest {
            skus: vec![sku1.clone(), sku2.clone()],
        };

        let request_bytes = request.encode_to_vec();
        let response = client
            .request("inventory.get_all_locations_by_sku", request_bytes.into())
            .await
            .expect("Failed to send get_all_locations_by_sku request");

        let get_response = InventoryGetAllLocationsBySkuResponse::decode(&*response.payload)
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

        // Should have 2 SKU summaries
        assert_eq!(get_response.sku_summaries.len(), 2);
        assert!(get_response.not_found_skus.is_empty());

        // Verify SKU1 aggregation (3 locations)
        let sku1_summary = get_response
            .sku_summaries
            .iter()
            .find(|s| s.sku == sku1)
            .expect("SKU1 summary not found");

        assert!(sku1_summary.total_inventory.is_some());
        let sku1_total = sku1_summary.total_inventory.as_ref().unwrap();
        assert_eq!(sku1_total.total_quantity, 180); // 100 + 50 + 30
        assert_eq!(sku1_total.total_reserved_quantity, 35); // 20 + 10 + 5
        assert_eq!(sku1_total.total_available_quantity, 145); // 180 - 35
        assert_eq!(sku1_total.location_count, 3);
        assert_eq!(sku1_total.min_stock_level_across_locations, 10);

        assert_eq!(sku1_summary.location_details.len(), 3);

        // Verify SKU2 aggregation (1 location)
        let sku2_summary = get_response
            .sku_summaries
            .iter()
            .find(|s| s.sku == sku2)
            .expect("SKU2 summary not found");

        assert!(sku2_summary.total_inventory.is_some());
        let sku2_total = sku2_summary.total_inventory.as_ref().unwrap();
        assert_eq!(sku2_total.total_quantity, 75);
        assert_eq!(sku2_total.total_reserved_quantity, 15);
        assert_eq!(sku2_total.total_available_quantity, 60);
        assert_eq!(sku2_total.location_count, 1);

        assert_eq!(sku2_summary.location_details.len(), 1);

        // Cleanup
        cleanup_inventory_item(&client, &sku1)
            .await
            .expect("Failed to cleanup SKU1");
        cleanup_inventory_item(&client, &sku2)
            .await
            .expect("Failed to cleanup SKU2");
    }

    #[tokio::test]
    async fn test_get_all_locations_by_sku_partial_results() {
        let client = setup_nats_client().await;

        // Create test inventory for only one SKU
        let existing_sku = format!(
            "TEST-EXISTING-{}",
            Uuid::new_v4().to_string()[0..8].to_uppercase()
        );
        let non_existing_sku = format!(
            "TEST-MISSING-{}",
            Uuid::new_v4().to_string()[0..8].to_uppercase()
        );

        create_test_inventory_item(&client, &existing_sku, "DC - Dallas, TX", 50, 10)
            .await
            .expect("Failed to create test inventory item");

        // Request both existing and non-existing SKUs
        let request = InventoryGetAllLocationsBySkuRequest {
            skus: vec![existing_sku.clone(), non_existing_sku.clone()],
        };

        let request_bytes = request.encode_to_vec();
        let response = client
            .request("inventory.get_all_locations_by_sku", request_bytes.into())
            .await
            .expect("Failed to send get_all_locations_by_sku request");

        let get_response = InventoryGetAllLocationsBySkuResponse::decode(&*response.payload)
            .expect("Failed to decode response");

        // Verify the response
        assert!(get_response.status.is_some());
        let status = get_response.status.unwrap();
        assert_eq!(status.code, Code::Ok as i32);

        // Should have 1 SKU summary and 1 not found
        assert_eq!(get_response.sku_summaries.len(), 1);
        assert_eq!(get_response.not_found_skus.len(), 1);

        // Verify found SKU
        let found_summary = &get_response.sku_summaries[0];
        assert_eq!(found_summary.sku, existing_sku);

        // Verify not found SKU
        assert_eq!(get_response.not_found_skus[0], non_existing_sku);

        // Cleanup
        cleanup_inventory_item(&client, &existing_sku)
            .await
            .expect("Failed to cleanup existing SKU");
    }

    #[tokio::test]
    async fn test_get_all_locations_by_sku_empty_skus() {
        let client = setup_nats_client().await;

        // Test with empty SKU list
        let request = InventoryGetAllLocationsBySkuRequest { skus: vec![] };

        let request_bytes = request.encode_to_vec();
        let response = client
            .request("inventory.get_all_locations_by_sku", request_bytes.into())
            .await
            .expect("Failed to send get_all_locations_by_sku request");

        let get_response = InventoryGetAllLocationsBySkuResponse::decode(&*response.payload)
            .expect("Failed to decode response");

        // Should return validation error
        assert!(get_response.status.is_some());
        let status = get_response.status.unwrap();
        assert_eq!(status.code, Code::InvalidArgument as i32);
        assert!(status.message.contains("cannot be empty"));
    }

    #[tokio::test]
    async fn test_get_all_locations_by_sku_too_many_skus() {
        let client = setup_nats_client().await;

        // Test with more than 100 SKUs
    let skus: Vec<String> = (0..101).map(|i| format!("SKU-{i}")).collect();

        let request = InventoryGetAllLocationsBySkuRequest { skus };

        let request_bytes = request.encode_to_vec();
        let response = client
            .request("inventory.get_all_locations_by_sku", request_bytes.into())
            .await
            .expect("Failed to send get_all_locations_by_sku request");

        let get_response = InventoryGetAllLocationsBySkuResponse::decode(&*response.payload)
            .expect("Failed to decode response");

        // Should return validation error
        assert!(get_response.status.is_some());
        let status = get_response.status.unwrap();
        assert_eq!(status.code, Code::InvalidArgument as i32);
        assert!(status.message.contains("Maximum 100 SKUs"));
    }

    #[tokio::test]
    async fn test_get_all_locations_by_sku_performance() {
        let client = setup_nats_client().await;

        // Create inventory items for performance test
        let mut test_skus = Vec::new();
        for i in 0..10 {
            let sku = format!("PERF-SKU-{i}");
            create_test_inventory_item(&client, &sku, "DC - Dallas, TX", 100, 20)
                .await
                .expect("Failed to create test inventory item");
            test_skus.push(sku);
        }

        // Measure response time
        let start = std::time::Instant::now();

        let request = InventoryGetAllLocationsBySkuRequest {
            skus: test_skus.clone(),
        };

        let request_bytes = request.encode_to_vec();
        let response = client
            .request("inventory.get_all_locations_by_sku", request_bytes.into())
            .await
            .expect("Failed to send get_all_locations_by_sku request");

        let get_response = InventoryGetAllLocationsBySkuResponse::decode(&*response.payload)
            .expect("Failed to decode response");

        let duration = start.elapsed();

        println!("Response time for 10 SKUs: {}ms", duration.as_millis());

        // Verify response is successful
        assert!(get_response.status.is_some());
        let status = get_response.status.unwrap();
        assert_eq!(status.code, Code::Ok as i32);

        // Should find all 10 SKUs
        assert_eq!(get_response.sku_summaries.len(), 10);
        assert!(get_response.not_found_skus.is_empty());

        // Assert reasonable performance (under 500ms for local testing)
        assert!(
            duration.as_millis() < 500,
            "Response time {}ms exceeds 500ms threshold",
            duration.as_millis()
        );

        // Cleanup
        for sku in &test_skus {
            cleanup_inventory_item(&client, sku)
                .await
                .expect("Failed to cleanup test SKU");
        }
    }

    #[tokio::test]
    async fn test_get_all_locations_by_sku_aggregation_accuracy() {
        let client = setup_nats_client().await;

        // Create test inventory with known values for aggregation testing
        let test_sku = format!(
            "AGGREGATION-TEST-{}",
            Uuid::new_v4().to_string()[0..8].to_uppercase()
        );

        // Create items with specific values for precise aggregation testing
        create_test_inventory_item(&client, &test_sku, "Location A", 100, 25)
            .await
            .expect("Failed to create test inventory item");
        create_test_inventory_item(&client, &test_sku, "Location B", 200, 50)
            .await
            .expect("Failed to create test inventory item");
        create_test_inventory_item(&client, &test_sku, "Location C", 150, 30)
            .await
            .expect("Failed to create test inventory item");

        let request = InventoryGetAllLocationsBySkuRequest {
            skus: vec![test_sku.clone()],
        };

        let request_bytes = request.encode_to_vec();
        let response = client
            .request("inventory.get_all_locations_by_sku", request_bytes.into())
            .await
            .expect("Failed to send get_all_locations_by_sku request");

        let get_response = InventoryGetAllLocationsBySkuResponse::decode(&*response.payload)
            .expect("Failed to decode response");

        // Verify response
        assert!(get_response.status.is_some());
        let status = get_response.status.unwrap();
        assert_eq!(status.code, Code::Ok as i32);

        assert_eq!(get_response.sku_summaries.len(), 1);
        let summary = &get_response.sku_summaries[0];

        // Verify precise aggregation
        assert!(summary.total_inventory.is_some());
        let total = summary.total_inventory.as_ref().unwrap();

        assert_eq!(total.total_quantity, 450); // 100 + 200 + 150
        assert_eq!(total.total_reserved_quantity, 105); // 25 + 50 + 30
        assert_eq!(total.total_available_quantity, 345); // 450 - 105
        assert_eq!(total.location_count, 3);
        assert_eq!(total.min_stock_level_across_locations, 10); // All created with min 10

        // Verify location details are present
        assert_eq!(summary.location_details.len(), 3);

        // Verify individual location details
        let locations: HashMap<String, &inventory_messages::InventoryLocationDetail> = summary
            .location_details
            .iter()
            .map(|detail| (detail.location.clone(), detail))
            .collect();

        assert!(locations.contains_key("Location A"));
        assert!(locations.contains_key("Location B"));
        assert!(locations.contains_key("Location C"));

        let loc_a = locations["Location A"];
        assert_eq!(loc_a.quantity, 100);
        assert_eq!(loc_a.reserved_quantity, 25);
        assert_eq!(loc_a.available_quantity, 75);

        // Cleanup
        cleanup_inventory_item(&client, &test_sku)
            .await
            .expect("Failed to cleanup test SKU");
    }
}
