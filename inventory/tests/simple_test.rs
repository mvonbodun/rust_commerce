#[cfg(test)]
mod tests {
    use chrono::Utc;
    use rust_inventory::model::InventoryItem;

    #[test]
    fn test_inventory_item_creation() {
        let now = Utc::now();
        let item = InventoryItem {
            id: Some("test-id".to_string()),
            sku: "TEST-SKU".to_string(),
            quantity: 100,
            reserved_quantity: 10,
            available_quantity: 90,
            min_stock_level: 20,
            location: "TEST_WAREHOUSE".to_string(),
            last_updated: now,
            created_at: now,
        };

        assert_eq!(item.sku, "TEST-SKU");
        assert_eq!(item.quantity, 100);
        assert_eq!(item.calculate_available_quantity(), 90);
        assert!(!item.is_low_stock());
    }

    #[test]
    fn test_low_stock_detection() {
        let now = Utc::now();
        let item = InventoryItem {
            id: Some("test-id".to_string()),
            sku: "LOW-STOCK-SKU".to_string(),
            quantity: 5,
            reserved_quantity: 0,
            available_quantity: 5,
            min_stock_level: 10,
            location: "TEST_WAREHOUSE".to_string(),
            last_updated: now,
            created_at: now,
        };

        assert!(item.is_low_stock());
    }
}
