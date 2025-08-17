use clap::{Parser, Subcommand};
use inventory_messages::{
    InventoryCreateRequest, InventoryCreateResponse, InventoryGetRequest, InventoryGetResponse,
    InventoryDeleteRequest, InventoryDeleteResponse, InventoryUpdateStockRequest, InventoryUpdateStockResponse,
    InventoryGetAllLocationsBySkuRequest, InventoryGetAllLocationsBySkuResponse,
};
use log::debug;
use prost::Message;
use serde::Deserialize;
use serde_json;
use std::fs;
use std::path::PathBuf;
use rust_inventory::InventoryItem;
use rust_common::load_environment;
use chrono::Utc;

pub mod inventory_messages {
    include!(concat!(env!("OUT_DIR"), "/inventory_messages.rs"));
}

// Structs for JSON import (handles string dates)
#[derive(Deserialize, Debug, Clone)]
pub struct InventoryItemImport {
    pub sku: String,
    pub quantity: i32,
    pub reserved_quantity: i32,
    pub min_stock_level: i32,
    pub location: String,
}

// Convert import struct to domain model
impl InventoryItemImport {
    fn to_inventory_item(&self) -> Result<InventoryItem, Box<dyn std::error::Error>> {
        let now = Utc::now();
        Ok(InventoryItem {
            id: None,
            sku: self.sku.clone(),
            quantity: self.quantity,
            reserved_quantity: self.reserved_quantity,
            available_quantity: self.quantity - self.reserved_quantity,
            min_stock_level: self.min_stock_level,
            location: self.location.clone(),
            last_updated: now,
            created_at: now,
        })
    }
}

#[derive(Parser)]
#[command(name = "inventory-client")]
#[command(about = "A CLI for managing inventory operations")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new inventory item
    Create {
        /// SKU of the product
        #[arg(short, long)]
        sku: String,
        /// Initial quantity
        #[arg(short, long)]
        quantity: i32,
        /// Reserved quantity
        #[arg(short, long, default_value = "0")]
        reserved_quantity: i32,
        /// Minimum stock level for alerts
        #[arg(short, long)]
        min_stock_level: i32,
        /// Storage location
        #[arg(short, long)]
        location: String,
    },
    /// Get inventory item by SKU
    Get {
        /// SKU of the product
        #[arg(short, long)]
        sku: String,
    },
    /// Delete inventory item by SKU
    Delete {
        /// SKU of the product
        #[arg(short, long)]
        sku: String,
    },
    /// Update stock levels
    UpdateStock {
        /// SKU of the product
        #[arg(short, long)]
        sku: String,
        /// Quantity change (positive or negative)
        #[arg(short, long)]
        quantity_change: i32,
        /// Reason for the change
        #[arg(short, long)]
        reason: String,
    },
    /// Get inventory across all locations for multiple SKUs
    GetMultiSku {
        /// Comma-separated list of SKUs (max 100)
        #[arg(short, long, value_delimiter = ',')]
        skus: Vec<String>,
    },
    /// Import inventory items from JSON file
    Import {
        /// Path to JSON file containing inventory items
        #[arg(short, long)]
        file: PathBuf,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    pretty_env_logger::init();
    
    // Load environment variables
    load_environment();
    
    let cli = Cli::parse();

    // Get NATS URL from environment
    let nats_url = std::env::var("NATS_URL").unwrap_or_else(|_| "nats://localhost:4222".to_string());

    // Connect to NATS
    let client = async_nats::connect(&nats_url).await?;

    match cli.command {
        Commands::Create { sku, quantity, reserved_quantity, min_stock_level, location } => {
            create_inventory_item(&client, sku, quantity, reserved_quantity, min_stock_level, location).await?;
        }
        Commands::Get { sku } => {
            get_inventory_item(&client, sku).await?;
        }
        Commands::Delete { sku } => {
            delete_inventory_item(&client, sku).await?;
        }
        Commands::UpdateStock { sku, quantity_change, reason } => {
            update_stock(&client, sku, quantity_change, reason).await?;
        }
        Commands::GetMultiSku { skus } => {
            get_multi_sku_inventory(&client, skus).await?;
        }
        Commands::Import { file } => {
            import_inventory_items(&client, file).await?;
        }
    }

    Ok(())
}

async fn create_inventory_item(
    client: &async_nats::Client,
    sku: String,
    quantity: i32,
    reserved_quantity: i32,
    min_stock_level: i32,
    location: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let request = InventoryCreateRequest {
        sku,
        quantity,
        reserved_quantity,
        min_stock_level,
        location,
    };

    let response = client
        .request("inventory.create_item", request.encode_to_vec().into())
        .await?;

    let response = InventoryCreateResponse::decode(response.payload)?;
    debug!("Create response: {:?}", response);

    match response.status {
        Some(status) if status.code == inventory_messages::Code::Ok as i32 => {
            if let Some(item) = response.item {
                println!("✓ Created inventory item:");
                println!("  SKU: {}", item.sku);
                println!("  Quantity: {}", item.quantity);
                println!("  Available: {}", item.available_quantity);
                println!("  Location: {}", item.location);
            }
        }
        Some(status) => {
            println!("✗ Failed to create inventory item: {}", status.message);
        }
        None => {
            println!("✗ Invalid response from server");
        }
    }

    Ok(())
}

async fn get_inventory_item(
    client: &async_nats::Client,
    sku: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let request = InventoryGetRequest { sku };

    let response = client
        .request("inventory.get_item", request.encode_to_vec().into())
        .await?;

    let response = InventoryGetResponse::decode(response.payload)?;
    debug!("Get response: {:?}", response);

    match response.status {
        Some(status) if status.code == inventory_messages::Code::Ok as i32 => {
            if let Some(item) = response.item {
                println!("✓ Found inventory item:");
                println!("  ID: {}", item.id.unwrap_or_default());
                println!("  SKU: {}", item.sku);
                println!("  Quantity: {}", item.quantity);
                println!("  Reserved: {}", item.reserved_quantity);
                println!("  Available: {}", item.available_quantity);
                println!("  Min Stock: {}", item.min_stock_level);
                println!("  Location: {}", item.location);
            }
        }
        Some(status) if status.code == inventory_messages::Code::NotFound as i32 => {
            println!("✗ Inventory item not found");
        }
        Some(status) => {
            println!("✗ Failed to get inventory item: {}", status.message);
        }
        None => {
            println!("✗ Invalid response from server");
        }
    }

    Ok(())
}

async fn delete_inventory_item(
    client: &async_nats::Client,
    sku: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let request = InventoryDeleteRequest { sku };

    let response = client
        .request("inventory.delete_item", request.encode_to_vec().into())
        .await?;

    let response = InventoryDeleteResponse::decode(response.payload)?;
    debug!("Delete response: {:?}", response);

    match response.status {
        Some(status) if status.code == inventory_messages::Code::Ok as i32 => {
            println!("✓ Inventory item deleted successfully");
        }
        Some(status) => {
            println!("✗ Failed to delete inventory item: {}", status.message);
        }
        None => {
            println!("✗ Invalid response from server");
        }
    }

    Ok(())
}

async fn update_stock(
    client: &async_nats::Client,
    sku: String,
    quantity_change: i32,
    reason: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let request = InventoryUpdateStockRequest {
        sku,
        quantity_change,
        reason,
    };

    let response = client
        .request("inventory.update_stock", request.encode_to_vec().into())
        .await?;

    let response = InventoryUpdateStockResponse::decode(response.payload)?;
    debug!("Update stock response: {:?}", response);

    match response.status {
        Some(status) if status.code == inventory_messages::Code::Ok as i32 => {
            if let Some(item) = response.item {
                println!("✓ Stock updated successfully:");
                println!("  SKU: {}", item.sku);
                println!("  New Quantity: {}", item.quantity);
                println!("  Available: {}", item.available_quantity);
            }
        }
        Some(status) if status.code == inventory_messages::Code::NotFound as i32 => {
            println!("✗ Inventory item not found");
        }
        Some(status) => {
            println!("✗ Failed to update stock: {}", status.message);
        }
        None => {
            println!("✗ Invalid response from server");
        }
    }

    Ok(())
}

async fn import_inventory_items(
    client: &async_nats::Client,
    file_path: PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    let file_content = fs::read_to_string(&file_path)?;
    let items: Vec<InventoryItemImport> = serde_json::from_str(&file_content)?;

    println!("Importing {} inventory items from {:?}", items.len(), file_path);

    for item_import in items {
        let item = item_import.to_inventory_item()?;
        
        let request = InventoryCreateRequest {
            sku: item.sku.clone(),
            quantity: item.quantity,
            reserved_quantity: item.reserved_quantity,
            min_stock_level: item.min_stock_level,
            location: item.location.clone(),
        };

        let response = client
            .request("inventory.create_item", request.encode_to_vec().into())
            .await?;

        let response = InventoryCreateResponse::decode(response.payload)?;

        match response.status {
            Some(status) if status.code == inventory_messages::Code::Ok as i32 => {
                println!("✓ Imported item: {}", item.sku);
            }
            Some(status) => {
                println!("✗ Failed to import item {}: {}", item.sku, status.message);
            }
            None => {
                println!("✗ Invalid response for item: {}", item.sku);
            }
        }
    }

    Ok(())
}

async fn get_multi_sku_inventory(
    client: &async_nats::Client,
    skus: Vec<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    if skus.is_empty() {
        println!("❌ No SKUs provided");
        return Ok(());
    }

    if skus.len() > 100 {
        println!("❌ Too many SKUs provided. Maximum is 100, got {}", skus.len());
        return Ok(());
    }

    println!("📦 Getting inventory for {} SKU(s)...", skus.len());

    let request = InventoryGetAllLocationsBySkuRequest {
        skus: skus.clone(),
    };

    let response = client
        .request("inventory.get_all_locations_by_sku", request.encode_to_vec().into())
        .await?;

    let response = InventoryGetAllLocationsBySkuResponse::decode(response.payload)?;

    match response.status {
        Some(status) if status.code == inventory_messages::Code::Ok as i32 => {
            println!("✅ Successfully retrieved inventory data\n");

            // Display found SKUs
            for sku_summary in &response.sku_summaries {
                println!("📋 SKU: {}", sku_summary.sku);
                
                if let Some(total) = &sku_summary.total_inventory {
                    println!("  📊 Total Inventory: {} units ({} available, {} reserved)",
                        total.total_quantity,
                        total.total_available_quantity,
                        total.total_reserved_quantity
                    );
                    println!("  📍 Locations: {}", total.location_count);
                    println!("  ⚠️  Min Stock Level: {}", total.min_stock_level_across_locations);
                }

                if !sku_summary.location_details.is_empty() {
                    println!("  🏪 Location Details:");
                    for detail in &sku_summary.location_details {
                        println!("    ├─ {}: {} units ({} available, {} reserved, min: {})",
                            detail.location,
                            detail.quantity,
                            detail.available_quantity,
                            detail.reserved_quantity,
                            detail.min_stock_level
                        );
                    }
                }
                println!();
            }

            // Display not found SKUs
            if !response.not_found_skus.is_empty() {
                println!("❌ SKUs not found:");
                for not_found_sku in &response.not_found_skus {
                    println!("  ✗ {}", not_found_sku);
                }
                println!();
            }

            // Summary
            println!("📈 Summary:");
            println!("  ✅ Found: {} SKUs", response.sku_summaries.len());
            println!("  ❌ Not found: {} SKUs", response.not_found_skus.len());
            println!("  📦 Total requested: {} SKUs", skus.len());
        }
        Some(status) => {
            println!("❌ Error: {} (Code: {})", status.message, status.code);
        }
        None => {
            println!("❌ Invalid response from server");
        }
    }

    Ok(())
}
