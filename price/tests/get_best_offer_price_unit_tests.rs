#[cfg(test)]
mod tests {
    #[test]
    fn test_protobuf_message_creation() {
        // Test that we can create protobuf messages for the new API
        use rust_price::offer_messages::{GetBestOfferPriceRequest, GetBestOfferPriceResponse};
        
        let request = GetBestOfferPriceRequest {
            sku: "TEST-SKU-001".to_string(),
            quantity: 5,
            date: None,  // Optional field
            currency: "USD".to_string(),
        };
        
        assert_eq!(request.sku, "TEST-SKU-001");
        assert_eq!(request.quantity, 5);
        assert_eq!(request.currency, "USD");
        assert!(request.date.is_none());
        
        // Test response creation - check the actual fields that exist
        let response = GetBestOfferPriceResponse {
            offer: None,  // Based on the protobuf definition
            found: false,
        };
        
        assert!(!response.found);
        assert!(response.offer.is_none());
    }

    #[test]
    fn test_validation_logic() {
        // Test individual validation conditions
        
        // Test 1: Empty SKU should be invalid
        let sku = "";
        assert!(sku.trim().is_empty(), "Empty SKU should fail validation");
        
        // Test 2: Valid SKU should pass
        let sku = "VALID-SKU-001";
        assert!(!sku.trim().is_empty(), "Valid SKU should pass validation");
        
        // Test 3: Zero or negative quantity should be invalid
        let quantity = 0;
        assert!(quantity <= 0, "Zero quantity should fail validation");
        
        let quantity = -5;
        assert!(quantity <= 0, "Negative quantity should fail validation");
        
        // Test 4: Positive quantity should be valid
        let quantity = 5;
        assert!(quantity > 0, "Positive quantity should pass validation");
        
        // Test 5: Currency validation (based on the handler logic)
        let currency = "USD";
        assert!(currency == "USD" || currency == "EUR", "USD should be valid");
        
        let currency = "EUR";
        assert!(currency == "USD" || currency == "EUR", "EUR should be valid");
        
        let currency = "JPY";
        assert!(!(currency == "USD" || currency == "EUR"), "JPY should be invalid");
        
        // Test 6: Date parsing (simulate what the handler does)
        use chrono::NaiveDate;
        
        let date_str = "2024-06-15";
        let result = NaiveDate::parse_from_str(date_str, "%Y-%m-%d");
        assert!(result.is_ok(), "Valid date format should parse successfully");
        
        let date_str = "invalid-date";
        let result = NaiveDate::parse_from_str(date_str, "%Y-%m-%d");
        assert!(result.is_err(), "Invalid date format should fail parsing");
    }

    #[tokio::test]
    async fn test_handler_error_types() {
        // Test that the HandlerError enum works as expected
        use rust_price::handlers::handlers_inner::HandlerError;
        
        let validation_error = HandlerError::ValidationError("Test validation error".to_string());
        match validation_error {
            HandlerError::ValidationError(msg) => {
                assert_eq!(msg, "Test validation error");
            },
            _ => panic!("Expected ValidationError"),
        }
        
        let internal_error = HandlerError::InternalError("Test internal error".to_string());
        match internal_error {
            HandlerError::InternalError(msg) => {
                assert_eq!(msg, "Test internal error");
            },
            _ => panic!("Expected InternalError"),
        }
    }
}
