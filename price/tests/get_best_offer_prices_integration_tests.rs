#[cfg(test)]
mod tests {
    use chrono::NaiveDate;
    use prost::Message;
    use rust_price::{
        model::Offer,
        offer_messages::{GetBestOfferPricesRequest, GetBestOfferPricesResponse, SkuOfferResult},
    };
    use std::collections::HashMap;

    #[test]
    fn test_protobuf_message_creation() {
        // Test that we can create protobuf messages for the multi-SKU API
        let request = GetBestOfferPricesRequest {
            skus: vec!["TEST-SKU-001".to_string(), "TEST-SKU-002".to_string()],
            quantity: 5,
            date: None,
            currency: "USD".to_string(),
        };

        assert_eq!(request.skus.len(), 2);
        assert_eq!(request.skus[0], "TEST-SKU-001");
        assert_eq!(request.skus[1], "TEST-SKU-002");
        assert_eq!(request.quantity, 5);
        assert_eq!(request.currency, "USD");
        assert!(request.date.is_none());

        // Test response creation
        let sku_result = SkuOfferResult {
            sku: "TEST-SKU-001".to_string(),
            offer: None,
            found: false,
        };

        let response = GetBestOfferPricesResponse {
            sku_results: vec![sku_result],
            status: None,
        };

        assert_eq!(response.sku_results.len(), 1);
        assert_eq!(response.sku_results[0].sku, "TEST-SKU-001");
        assert!(!response.sku_results[0].found);
        assert!(response.sku_results[0].offer.is_none());
    }

    #[test]
    fn test_validation_logic() {
        // Test empty SKUs list
        let skus: Vec<String> = vec![];
        assert!(skus.is_empty(), "Empty SKUs list should fail validation");

        // Test too many SKUs (over 100)
        let skus: Vec<String> = (0..101).map(|i| format!("SKU-{i:03}")).collect();
        assert!(skus.len() > 100, "Over 100 SKUs should fail validation");

        // Test valid SKUs list
        let skus = vec!["SKU-001".to_string(), "SKU-002".to_string()];
        assert!(
            !skus.is_empty() && skus.len() <= 100,
            "Valid SKUs list should pass"
        );

        // Test SKU with empty string
        let skus = vec!["SKU-001".to_string(), "".to_string()];
        let has_empty = skus.iter().any(|s| s.trim().is_empty());
        assert!(has_empty, "List with empty SKU should fail validation");

        // Test currency validation
        let currency = "USD";
        assert!(
            currency == "USD" || currency == "EUR",
            "Valid currency should pass"
        );

        let currency = "GBP";
        assert!(
            !(currency == "USD" || currency == "EUR"),
            "Invalid currency should fail"
        );

        // Test quantity validation
        let quantity = 5;
        assert!(quantity > 0, "Positive quantity should pass validation");

        let quantity = 0;
        assert!(quantity <= 0, "Zero quantity should fail validation");
    }

    #[test]
    fn test_protobuf_encoding_decoding() {
        // Test that we can encode and decode the protobuf messages correctly
        let original_request = GetBestOfferPricesRequest {
            skus: vec!["TEST-SKU-001".to_string(), "TEST-SKU-002".to_string()],
            quantity: 10,
            date: Some("2024-12-01".to_string()),
            currency: "EUR".to_string(),
        };

        // Encode to bytes
        let encoded = original_request.encode_to_vec();
        assert!(!encoded.is_empty(), "Encoded message should not be empty");

        // Decode back to message
        let decoded_request = GetBestOfferPricesRequest::decode(encoded.as_slice())
            .expect("Should be able to decode the message");

        assert_eq!(decoded_request.skus, original_request.skus);
        assert_eq!(decoded_request.quantity, original_request.quantity);
        assert_eq!(decoded_request.date, original_request.date);
        assert_eq!(decoded_request.currency, original_request.currency);
    }

    #[tokio::test]
    async fn test_handler_error_types() {
        use async_trait::async_trait;
        use rust_price::handlers::handlers_inner::{get_best_offer_prices, HandlerError};
        use rust_price::persistence::offer_dao::OfferDao;

        // Mock DAO that always returns an error
        struct MockErrorDao;

        #[async_trait]
        impl OfferDao for MockErrorDao {
            async fn create_offer(
                &self,
                _offer: Offer,
            ) -> Result<Offer, rust_price::model::DBError> {
                unimplemented!()
            }

            async fn delete_offer(
                &self,
                _offer_id: String,
            ) -> Result<(), rust_price::model::DBError> {
                unimplemented!()
            }

            async fn get_offer(
                &self,
                _offer_id: String,
            ) -> Result<Option<Offer>, rust_price::model::DBError> {
                unimplemented!()
            }

            async fn find_best_offer_price(
                &self,
                _sku: &str,
                _quantity: i32,
                _date: NaiveDate,
                _currency: &str,
            ) -> Result<Option<Offer>, rust_price::model::DBError> {
                unimplemented!()
            }

            async fn find_best_offer_prices(
                &self,
                _skus: &[String],
                _quantity: i32,
                _date: NaiveDate,
                _currency: &str,
            ) -> Result<HashMap<String, Option<Offer>>, rust_price::model::DBError> {
                Err(rust_price::model::DBError::Other(Box::new(
                    std::io::Error::other("Mock error"),
                )))
            }
        }

        let mock_dao = MockErrorDao;

        // Test validation errors
        let result = get_best_offer_prices(
            vec![], // Empty SKUs list
            5,
            None,
            "USD".to_string(),
            &mock_dao,
        )
        .await;

        match result {
            Err(HandlerError::ValidationError(msg)) => {
                assert!(
                    msg.contains("empty"),
                    "Should be a validation error about empty SKUs"
                );
            }
            _ => panic!("Expected ValidationError for empty SKUs list"),
        }

        // Test too many SKUs
        let many_skus: Vec<String> = (0..101).map(|i| format!("SKU-{i:03}")).collect();
        let result = get_best_offer_prices(many_skus, 5, None, "USD".to_string(), &mock_dao).await;

        match result {
            Err(HandlerError::ValidationError(msg)) => {
                assert!(
                    msg.contains("Too many SKUs"),
                    "Should be a validation error about too many SKUs"
                );
            }
            _ => panic!("Expected ValidationError for too many SKUs"),
        }

        // Test internal error (database error)
        let result = get_best_offer_prices(
            vec!["TEST-SKU-001".to_string()],
            5,
            None,
            "USD".to_string(),
            &mock_dao,
        )
        .await;

        match result {
            Err(HandlerError::InternalError(msg)) => {
                assert!(
                    msg.contains("Failed to find"),
                    "Should be an internal error from DAO"
                );
            }
            _ => panic!("Expected InternalError from mock DAO"),
        }
    }

    #[test]
    fn test_sku_parsing_and_deduplication() {
        // Test parsing comma-separated SKUs (simulating CLI input parsing)
        let sku_input = "SKU-001,SKU-002, SKU-003 ,SKU-004";
        let parsed_skus: Vec<String> = sku_input
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        assert_eq!(parsed_skus.len(), 4);
        assert_eq!(parsed_skus[0], "SKU-001");
        assert_eq!(parsed_skus[1], "SKU-002");
        assert_eq!(parsed_skus[2], "SKU-003");
        assert_eq!(parsed_skus[3], "SKU-004");

        // Test with empty values
        let sku_input = "SKU-001,,SKU-002, ,SKU-003";
        let parsed_skus: Vec<String> = sku_input
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        assert_eq!(parsed_skus.len(), 3);
        assert_eq!(parsed_skus[0], "SKU-001");
        assert_eq!(parsed_skus[1], "SKU-002");
        assert_eq!(parsed_skus[2], "SKU-003");

        // Test deduplication (in case CLI needs it)
        let mut skus = vec![
            "SKU-001".to_string(),
            "SKU-002".to_string(),
            "SKU-001".to_string(),
        ];
        skus.sort();
        skus.dedup();

        assert_eq!(skus.len(), 2);
        assert_eq!(skus[0], "SKU-001");
        assert_eq!(skus[1], "SKU-002");
    }

    #[test]
    fn test_response_aggregation_logic() {
        // Test the logic that would be used to aggregate results from multiple SKUs
        let mut results = HashMap::new();
        results.insert("SKU-001".to_string(), Some(create_mock_offer("SKU-001")));
        results.insert("SKU-002".to_string(), None);
        results.insert("SKU-003".to_string(), Some(create_mock_offer("SKU-003")));

        // Count found vs not found
        let found_count = results.values().filter(|opt| opt.is_some()).count();
        let not_found_count = results.values().filter(|opt| opt.is_none()).count();

        assert_eq!(found_count, 2);
        assert_eq!(not_found_count, 1);
        assert_eq!(results.len(), 3);

        // Test conversion to SkuOfferResult
        let sku_results: Vec<SkuOfferResult> = results
            .into_iter()
            .map(|(sku, offer_option)| {
                let found = offer_option.is_some();
                SkuOfferResult {
                    sku: sku.clone(),
                    offer: offer_option.map(|_| rust_price::offer_messages::Offer {
                        id: Some("test-id".to_string()),
                        sku: sku.clone(),
                        start_date: None,
                        end_date: None,
                        min_quantity: 1,
                        max_quantity: None,
                        offer_prices: vec![],
                    }),
                    found,
                }
            })
            .collect();

        assert_eq!(sku_results.len(), 3);
        let found_results: Vec<_> = sku_results.iter().filter(|r| r.found).collect();
        assert_eq!(found_results.len(), 2);
    }

    fn create_mock_offer(sku: &str) -> Offer {
        use bson::Decimal128;
        use chrono::{DateTime, Utc};
        use iso_currency::Currency;
        use rust_price::model::OfferPrice;
        use std::str::FromStr;

        Offer {
            id: Some("test-id".to_string()),
            sku: sku.to_string(),
            start_date: DateTime::parse_from_rfc3339("2024-01-01T00:00:00Z")
                .unwrap()
                .with_timezone(&Utc),
            end_date: DateTime::parse_from_rfc3339("2024-12-31T23:59:59Z")
                .unwrap()
                .with_timezone(&Utc),
            min_quantity: 1,
            max_quantity: Some(100),
            offer_prices: vec![OfferPrice {
                price: Decimal128::from_str("10.00").unwrap(),
                currency: Currency::from_code("USD").unwrap(),
            }],
        }
    }
}
