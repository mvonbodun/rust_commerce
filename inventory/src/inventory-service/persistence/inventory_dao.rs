use log::{debug, error, info};
use std::collections::HashMap;

use async_trait::async_trait;
use bson::doc;
use mongodb::Collection;

use crate::model::{DBError, InventoryItem};

#[async_trait]
pub trait InventoryDao {
    async fn create_item(&self, item: InventoryItem) -> Result<InventoryItem, DBError>;
    async fn delete_item(&self, sku: String) -> Result<(), DBError>;
    async fn get_item(&self, sku: String) -> Result<Option<InventoryItem>, DBError>;
    async fn get_items_by_skus(
        &self,
        skus: Vec<String>,
    ) -> Result<HashMap<String, Vec<InventoryItem>>, DBError>;
    async fn update_stock(
        &self,
        sku: String,
        quantity_change: i32,
        reason: String,
    ) -> Result<Option<InventoryItem>, DBError>;
    #[allow(dead_code)]
    async fn find_low_stock_items(
        &self,
        location: Option<String>,
    ) -> Result<Vec<InventoryItem>, DBError>;
}

pub struct InventoryDaoImpl {
    collection: Collection<InventoryItem>,
}

impl InventoryDaoImpl {
    pub fn new(collection: Collection<InventoryItem>) -> Self {
        InventoryDaoImpl { collection }
    }
}

#[async_trait]
impl InventoryDao for InventoryDaoImpl {
    // Create an inventory item
    async fn create_item(&self, item: InventoryItem) -> Result<InventoryItem, DBError> {
        let insert_result = self.collection.insert_one(&item).await.map_err(|error| {
            error!("Error on insert: {error:?}");
            DBError::Other(Box::new(error))
        })?;

        info!("Inserted inventory item result: {insert_result:?}");
        debug!("Inventory item after insert: {item:?}");
        Ok(item)
    }

    // Get an inventory item by SKU
    async fn get_item(&self, sku: String) -> Result<Option<InventoryItem>, DBError> {
        debug!("before call to find_one - sku: {sku:?}");
        let find_result = self
            .collection
            .find_one(doc! {"sku": &sku})
            .await
            .map_err(|error| {
                error!("DB error: {error:?}");
                DBError::Other(Box::new(error))
            })?;

        match find_result {
            Some(item) => {
                debug!("Found inventory item: {item:?}");
                Ok(Some(item))
            }
            None => {
                debug!("Inventory item not found for sku: {sku:?}");
                Ok(None)
            }
        }
    }

    // Get inventory items for multiple SKUs across all locations
    async fn get_items_by_skus(
        &self,
        skus: Vec<String>,
    ) -> Result<HashMap<String, Vec<InventoryItem>>, DBError> {
        debug!("Getting inventory items for {} SKUs: {skus:?}", skus.len());

        if skus.is_empty() {
            return Ok(HashMap::new());
        }

        // Query for all items with SKUs in the provided list
        let query = doc! {"sku": {"$in": &skus}};

        let mut cursor = self.collection.find(query).await.map_err(|error| {
            error!("DB error in get_items_by_skus: {error:?}");
            DBError::Other(Box::new(error))
        })?;

        let mut result: HashMap<String, Vec<InventoryItem>> = HashMap::new();
        use futures::stream::StreamExt;

        while let Some(item_result) = cursor.next().await {
            match item_result {
                Ok(item) => {
                    debug!("Found inventory item: {} at {}", item.sku, item.location);
                    result.entry(item.sku.clone()).or_default().push(item);
                }
                Err(error) => {
                    error!("DB cursor error in get_items_by_skus: {error:?}");
                    return Err(DBError::Other(Box::new(error)));
                }
            }
        }

        debug!(
            "Found inventory for {} SKUs out of {} requested",
            result.len(),
            skus.len()
        );
        Ok(result)
    }

    // Delete an inventory item
    async fn delete_item(&self, sku: String) -> Result<(), DBError> {
        let delete_result = self
            .collection
            .delete_one(doc! {"sku": &sku})
            .await
            .map_err(|error| {
                error!("Error on delete: {error:?}");
                DBError::Other(Box::new(error))
            })?;

        info!("Deleted inventory item result: {delete_result:?}");
        Ok(())
    }

    // Update stock levels for an item
    async fn update_stock(
        &self,
        sku: String,
        quantity_change: i32,
        reason: String,
    ) -> Result<Option<InventoryItem>, DBError> {
        debug!(
            "Updating stock for sku: {}, quantity_change: {}, reason: {}",
            sku, quantity_change, reason
        );

        // First, get the current item
        let current_item = self.get_item(sku.clone()).await?;

        match current_item {
            Some(mut item) => {
                // Update quantities
                item.quantity += quantity_change;
                item.available_quantity = item.quantity - item.reserved_quantity;
                item.last_updated = chrono::Utc::now();

                // Update the document in MongoDB
                let update_doc = doc! {
                    "$set": {
                        "quantity": item.quantity,
                        "available_quantity": item.available_quantity,
                        "last_updated": bson::DateTime::from_chrono(item.last_updated)
                    }
                };

                let update_result = self
                    .collection
                    .update_one(doc! {"sku": &sku}, update_doc)
                    .await
                    .map_err(|error| {
                        error!("Error on update: {error:?}");
                        DBError::Other(Box::new(error))
                    })?;

                info!("Updated inventory item result: {update_result:?}");
                Ok(Some(item))
            }
            None => {
                debug!("Inventory item not found for sku: {sku}");
                Ok(None)
            }
        }
    }

    // Find items with low stock
    async fn find_low_stock_items(
        &self,
        location: Option<String>,
    ) -> Result<Vec<InventoryItem>, DBError> {
        debug!("Finding low stock items for location: {location:?}");

        let mut query = doc! {
            "$expr": {
                "$lte": ["$available_quantity", "$min_stock_level"]
            }
        };

        // Add location filter if provided
        if let Some(loc) = location {
            query.insert("location", loc);
        }

        debug!("MongoDB query for low stock: {query:?}");

        let mut cursor = self.collection.find(query).await.map_err(|error| {
            error!("DB error in find_low_stock_items: {error:?}");
            DBError::Other(Box::new(error))
        })?;

        let mut items = Vec::new();
        use futures::stream::StreamExt;

        while let Some(result) = cursor.next().await {
            match result {
                Ok(item) => {
                    debug!("Found low stock item: {item:?}");
                    items.push(item);
                }
                Err(error) => {
                    error!("DB cursor error in find_low_stock_items: {error:?}");
                    return Err(DBError::Other(Box::new(error)));
                }
            }
        }

        debug!("Found {} low stock items", items.len());
        Ok(items)
    }
}
