use log::{debug, info, warn};
use rust_common::{validate_dependencies, ErrorContext};

/// Service-specific dependency validation for price service
pub async fn validate_price_dependencies(
    mongo_client: &mongodb::Client,
    nats_client: &async_nats::Client,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // First run generic validation from rust_common
    validate_dependencies(mongo_client, nats_client).await?;

    // Price-specific validation
    let database = mongo_client.database("db_prices");
    let collections = database
        .list_collection_names()
        .await
        .with_context("Failed to list price collections")?;

    for required_collection in &["prices", "offers"] {
        if collections.contains(&required_collection.to_string()) {
            debug!("✅ Collection '{required_collection}' exists");
        } else {
            warn!("⚠️  Collection '{required_collection}' not found, will be created on first use");
        }
    }

    info!("✅ Price-specific dependencies validated");
    Ok(())
}
