use once_cell::sync::Lazy;
use std::sync::Arc;
use tokio::sync::Mutex;
use rust_catalog::startup::{Application, Settings};
use rust_common::{load_environment, test_helpers::{TestApp, TestConfig}};
use uuid::Uuid;
use log::info;

// Shared application instance for all tests
static SHARED_APP: Lazy<Arc<Mutex<Option<Arc<Application>>>>> = Lazy::new(|| {
    Arc::new(Mutex::new(None))
});

/// Get or create a shared application instance
/// This ensures only one catalog service is running for all tests
pub async fn get_shared_app() -> TestApp {
    // Generate a unique test database name for this test
    let db_name = format!(
        "test_catalog_{}",
        Uuid::new_v4().to_string().replace("-", "_")
    );
    info!("🧪 Using test database: {db_name}");

    // Load environment and initialize logger once
    static INIT: std::sync::Once = std::sync::Once::new();
    INIT.call_once(|| {
        load_environment();
        let _ = pretty_env_logger::try_init();
    });

    // Get settings for this test's database
    let settings = Settings::from_env();
    let mongodb_url = settings.mongodb_url.clone();
    let nats_url = settings.nats_url.clone();

    // Create a TestApp that connects to the shared catalog service
    TestApp::new_with_config(TestConfig {
        nats_url,
        mongodb_url,
        test_db_name: db_name,
    })
    .await
}