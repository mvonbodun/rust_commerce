#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_model_creation() {
        // Simple test to verify the build works
        use rust_price::model::{Offer, OfferPrice};
        use chrono::{DateTime, Utc};
        use bson::Decimal128;
        use iso_currency::Currency;
        
        use std::str::FromStr;
        
        let offer = Offer {
            id: Some("test".to_string()),
            sku: "TEST-001".to_string(),
            start_date: DateTime::parse_from_rfc3339("2024-01-01T00:00:00Z").unwrap().with_timezone(&Utc),
            end_date: DateTime::parse_from_rfc3339("2024-12-31T23:59:59Z").unwrap().with_timezone(&Utc),
            min_quantity: 1,
            max_quantity: Some(10),
            offer_prices: vec![
                OfferPrice {
                    currency: Currency::USD,
                    price: Decimal128::from_str("19.99").unwrap(),
                }
            ],
        };
        
        assert_eq!(offer.sku, "TEST-001");
        assert_eq!(offer.min_quantity, 1);
        assert_eq!(offer.offer_prices.len(), 1);
        assert_eq!(offer.offer_prices[0].currency, Currency::USD);
    }
}
