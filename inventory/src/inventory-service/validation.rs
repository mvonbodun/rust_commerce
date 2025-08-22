use log::{debug, info, warn};
use rust_common::{validate_dependencies, ErrorContext};

/// Service-specific dependency validation for inventory service
pub async fn validate_inventory_dependencies(
    mongo_client: &mongodb::Client,
    nats_client: &async_nats::Client,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // First run generic validation from rust_common
    validate_dependencies(mongo_client, nats_client).await?;

    // Inventory-specific validation
    let database = mongo_client.database("db_inventory");
    let collections = database
        .list_collection_names()
        .await
        .with_context("Failed to list inventory collections")?;

    for required_collection in &["inventory"] {
        if collections.contains(&required_collection.to_string()) {
            debug!("✅ Collection '{required_collection}' exists");
        } else {
            warn!("⚠️  Collection '{required_collection}' not found, will be created on first use");
        }
    }

    info!("✅ Inventory-specific dependencies validated");
    Ok(())
}
