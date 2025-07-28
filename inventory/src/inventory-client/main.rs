use clap::{Parser, Subcommand};
use inventory_messages::{
    InventoryCreateRequest, InventoryCreateResponse, InventoryGetRequest, InventoryGetResponse,
    InventoryDeleteRequest, InventoryDeleteResponse, InventoryUpdateStockRequest, InventoryUpdateStockResponse,
};
use log::debug;
use prost::Message;
use serde::Deserialize;
use serde_json;
use std::fs;
use std::path::PathBuf;
use rust_inventory::InventoryItem;
use chrono::{DateTime, Utc};

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
    
    let cli = Cli::parse();

    // Connect to NATS
    let client = async_nats::connect("0.0.0.0:4222").await?;

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
