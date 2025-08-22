use log::{debug, info, warn};
use rust_common::{validate_dependencies, ErrorContext};

/// Service-specific dependency validation for catalog service
pub async fn validate_catalog_dependencies(
    mongo_client: &mongodb::Client,
    nats_client: &async_nats::Client,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // First run generic validation from rust_common
    validate_dependencies(mongo_client, nats_client).await?;

    // Catalog-specific validation
    let database = mongo_client.database("db_catalog");
    let collections = database
        .list_collection_names()
        .await
        .with_context("Failed to list catalog collections")?;

    for required_collection in &["products", "categories", "category_tree_cache"] {
        if collections.contains(&required_collection.to_string()) {
            debug!("✅ Collection '{required_collection}' exists");
        } else {
            warn!("⚠️  Collection '{required_collection}' not found, will be created on first use");
        }
    }

    info!("✅ Catalog-specific dependencies validated");
    Ok(())
}
