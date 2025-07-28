use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InventoryItem {
    #[serde(rename = "_id")]
    pub id: Option<String>,
    pub sku: String,
    pub quantity: i32,
    pub reserved_quantity: i32,
    pub available_quantity: i32,
    pub min_stock_level: i32,
    pub location: String,
    #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub last_updated: DateTime<Utc>,
    #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub created_at: DateTime<Utc>,
}

impl InventoryItem {
    pub fn builder() -> InventoryItemBuilder {
        InventoryItemBuilder::default()
    }

    pub fn calculate_available_quantity(&self) -> i32 {
        self.quantity - self.reserved_quantity
    }

    pub fn is_low_stock(&self) -> bool {
        self.available_quantity <= self.min_stock_level
    }
}

#[derive(Default)]
pub struct InventoryItemBuilder {
    id: Option<String>,
    sku: String,
    quantity: i32,
    reserved_quantity: i32,
    available_quantity: i32,
    min_stock_level: i32,
    location: String,
    last_updated: DateTime<Utc>,
    created_at: DateTime<Utc>,
}

impl InventoryItemBuilder {
    pub fn new(
        sku: String,
        quantity: i32,
        min_stock_level: i32,
        location: String,
    ) -> Self {
        let now = Utc::now();
        let available_quantity = quantity; // Initially no reservations
        
        InventoryItemBuilder {
            id: Some(Uuid::new_v4().to_string()),
            sku,
            quantity,
            reserved_quantity: 0,
            available_quantity,
            min_stock_level,
            location,
            last_updated: now,
            created_at: now,
        }
    }

    pub fn sku(&mut self, sku: String) -> &mut Self {
        self.sku = sku;
        self
    }

    pub fn quantity(&mut self, quantity: i32) -> &mut Self {
        self.quantity = quantity;
        self.available_quantity = quantity - self.reserved_quantity;
        self
    }

    pub fn reserved_quantity(&mut self, reserved_quantity: i32) -> &mut Self {
        self.reserved_quantity = reserved_quantity;
        self.available_quantity = self.quantity - reserved_quantity;
        self
    }

    pub fn min_stock_level(&mut self, min_stock_level: i32) -> &mut Self {
        self.min_stock_level = min_stock_level;
        self
    }

    pub fn location(&mut self, location: String) -> &mut Self {
        self.location = location;
        self
    }

    pub fn build(&mut self) -> InventoryItem {
        InventoryItem {
            id: self.id.clone(),
            sku: self.sku.clone(),
            quantity: self.quantity,
            reserved_quantity: self.reserved_quantity,
            available_quantity: self.available_quantity,
            min_stock_level: self.min_stock_level,
            location: self.location.clone(),
            last_updated: self.last_updated,
            created_at: self.created_at,
        }
    }
}

#[derive(Debug, Error)]
pub enum DBError {
    #[error("Database connection error")]
    Connection,
    #[error("Database query error")]
    Query,
    #[error("Database transaction error")]
    Transaction,
    #[error("Database error occurred")]
    Other(#[from] Box<dyn std::error::Error + Send + Sync>),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inventory_item_builder_test() {
        let item = InventoryItemBuilder::new(
            "SKU123".to_string(),
            100,
            10,
            "WAREHOUSE_A".to_string(),
        )
        .build();
        
        assert_eq!(item.sku, "SKU123");
        assert_eq!(item.quantity, 100);
        assert_eq!(item.available_quantity, 100);
        assert_eq!(item.min_stock_level, 10);
        assert!(!item.is_low_stock());
        println!("Inventory Item: {:?}", item);
    }

    #[test]
    fn inventory_item_low_stock_test() {
        let item = InventoryItemBuilder::new(
            "SKU456".to_string(),
            5,
            10,
            "WAREHOUSE_B".to_string(),
        )
        .build();
        
        assert!(item.is_low_stock());
    }
}
