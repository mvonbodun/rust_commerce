use std::collections::HashMap;
use log::{debug, error};

use crate::model::InventoryItem;
use crate::persistence::inventory_dao::InventoryDao;

pub enum HandlerError {
    InternalError(String),
    ValidationError(String),
}

pub async fn create_item(
    item: InventoryItem,
    inventory_dao: &(dyn InventoryDao + Sync + Send),
) -> Result<InventoryItem, HandlerError> {
    debug!("Before call to create_item");

    let result = inventory_dao.create_item(item).await;
    match result {
        Ok(item) => Ok(item),
        Err(e) => {
            error!("Error creating inventory item: {}", e);
            Err(HandlerError::InternalError(format!(
                "Failed to create inventory item: {}",
                e
            )))
        }
    }
}

pub async fn get_item(
    sku: String,
    inventory_dao: &(dyn InventoryDao + Sync + Send),
) -> Result<Option<InventoryItem>, HandlerError> {
    debug!("Before call to get inventory item");
    let result = inventory_dao.get_item(sku).await;
    debug!("After call to get inventory item: {:?}", result);

    match result {
        Ok(Some(item)) => Ok(Some(item)),
        Ok(None) => Ok(None),
        Err(e) => {
            error!("Error getting inventory item: {}", e);
            Err(HandlerError::InternalError(format!(
                "Failed to get inventory item: {}",
                e
            )))
        }
    }
}

pub async fn delete_item(
    sku: String,
    inventory_dao: &(dyn InventoryDao + Sync + Send),
) -> Result<(), HandlerError> {
    debug!("Before call to delete inventory item");
    let result = inventory_dao.delete_item(sku).await;
    debug!("After call to delete inventory item: {:?}", result);

    match result {
        Ok(_) => Ok(()),
        Err(e) => {
            error!("Error deleting inventory item: {}", e);
            Err(HandlerError::InternalError(format!(
                "Failed to delete inventory item: {}",
                e
            )))
        }
    }
}

pub async fn update_stock(
    sku: String,
    quantity_change: i32,
    reason: String,
    inventory_dao: &(dyn InventoryDao + Sync + Send),
) -> Result<Option<InventoryItem>, HandlerError> {
    debug!("Before call to update stock");
    let result = inventory_dao.update_stock(sku, quantity_change, reason).await;
    debug!("After call to update stock: {:?}", result);

    match result {
        Ok(item) => Ok(item),
        Err(e) => {
            error!("Error updating stock: {}", e);
            Err(HandlerError::InternalError(format!(
                "Failed to update stock: {}",
                e
            )))
        }
    }
}

pub async fn get_all_locations_by_sku(
    skus: Vec<String>,
    inventory_dao: &(dyn InventoryDao + Sync + Send),
) -> Result<(HashMap<String, Vec<InventoryItem>>, Vec<String>), HandlerError> {
    debug!("Before call to get_items_by_skus for {} SKUs", skus.len());
    
    if skus.is_empty() {
        return Ok((HashMap::new(), Vec::new()));
    }
    
    let result = inventory_dao.get_items_by_skus(skus.clone()).await;
    debug!("After call to get_items_by_skus: {:?}", result.is_ok());
    
    match result {
        Ok(inventory_by_sku) => {
            // Determine which SKUs were not found
            let found_skus: std::collections::HashSet<String> = inventory_by_sku.keys().cloned().collect();
            let requested_skus: std::collections::HashSet<String> = skus.into_iter().collect();
            let not_found_skus: Vec<String> = requested_skus.difference(&found_skus).cloned().collect();
            
            debug!("Found {} SKUs, {} not found", found_skus.len(), not_found_skus.len());
            Ok((inventory_by_sku, not_found_skus))
        }
        Err(e) => {
            error!("Error getting inventory items by SKUs: {}", e);
            Err(HandlerError::InternalError(format!(
                "Failed to get inventory items by SKUs: {}",
                e
            )))
        }
    }
}
