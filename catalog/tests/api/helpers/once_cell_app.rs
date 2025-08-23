// Alternative test infrastructure using a global shared app instance
// Currently not used - tests use spawn_app.rs which creates a new instance per test
// This approach could be useful for faster test execution by sharing a single service
// across all tests, but requires careful test isolation

use mongodb::Client;
use once_cell::sync::Lazy;
use rust_common::test_helpers::TestApp;
use std::process::{Child, Command};
use std::sync::Mutex;
use uuid::Uuid;

pub struct GlobalTestApp {
    pub db_name: String,
    pub mongodb_client: Client,
    catalog_process: Option<Child>,
}

// Global test app instance that's initialized once
pub static TEST_APP: Lazy<Mutex<Option<GlobalTestApp>>> = Lazy::new(|| Mutex::new(None));

/// Initialize the test app once for all tests
pub async fn init_test_app() -> (TestApp, String, Client) {
    let mut guard = TEST_APP.lock().unwrap();

    if let Some(ref test_app) = *guard {
        // Already initialized, create new TestApp instance that connects to same NATS
        let app = TestApp::spawn().await;
        return (
            app,
            test_app.db_name.clone(),
            test_app.mongodb_client.clone(),
        );
    }

    // First time initialization
    eprintln!("🚀 Initializing global test environment...");

    // Generate a unique test database name for this test run
    let db_name = format!(
        "test_catalog_{}",
        Uuid::new_v4().to_string().replace("-", "_")
    );
    eprintln!("📦 Using test database: {db_name}");

    // Kill any existing catalog service
    Command::new("pkill")
        .args(["-f", "catalog-service"])
        .output()
        .ok();

    // Wait for the service to stop
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    // Start a new catalog service with the test database
    let catalog_process = Command::new("cargo")
        .args(["run", "--bin", "catalog-service"])
        .env("CATALOG_DB_NAME", &db_name)
        .env(
            "MONGODB_URL",
            std::env::var("MONGODB_URL")
                .unwrap_or_else(|_| "mongodb://admin:password123@localhost:27017".to_string()),
        )
        .env(
            "NATS_URL",
            std::env::var("NATS_URL").unwrap_or_else(|_| "nats://localhost:4222".to_string()),
        )
        .env("RUST_LOG", "info")
        .spawn()
        .expect("Failed to start catalog service");

    eprintln!("⏳ Waiting for catalog service to start (this may take a while on first run)...");

    // Wait longer for first compilation and startup
    tokio::time::sleep(std::time::Duration::from_secs(10)).await;

    // Create a TestApp for making requests
    let app = TestApp::spawn().await;

    // Connect to MongoDB
    let mongodb_client = Client::with_uri_str(
        &std::env::var("MONGODB_URL")
            .unwrap_or_else(|_| "mongodb://admin:password123@localhost:27017".to_string()),
    )
    .await
    .expect("Failed to connect to MongoDB");

    // Store the global instance
    *guard = Some(GlobalTestApp {
        db_name: db_name.clone(),
        mongodb_client: mongodb_client.clone(),
        catalog_process: Some(catalog_process),
    });

    eprintln!("✅ Global test environment ready with database: {db_name}");

    (app, db_name, mongodb_client)
}

/// Get the test app (initializes if needed)
pub async fn get_test_app() -> (TestApp, String, Client) {
    init_test_app().await
}

/// Cleanup helper to clear all products
pub async fn cleanup_products(client: &Client, db_name: &str) {
    let db = client.database(db_name);
    let collection = db.collection::<bson::Document>("products");
    if let Ok(result) = collection.delete_many(bson::doc! {}).await {
        if result.deleted_count > 0 {
            eprintln!("  🧹 Cleaned up {} products", result.deleted_count);
        }
    }
}

/// Cleanup helper to clear all categories  
pub async fn cleanup_categories(client: &Client, db_name: &str) {
    let db = client.database(db_name);
    let collection = db.collection::<bson::Document>("categories");
    if let Ok(result) = collection.delete_many(bson::doc! {}).await {
        if result.deleted_count > 0 {
            eprintln!("  🧹 Cleaned up {} categories", result.deleted_count);
        }
    }
}

/// Cleanup all test data
pub async fn cleanup_all(client: &Client, db_name: &str) {
    cleanup_products(client, db_name).await;
    cleanup_categories(client, db_name).await;
}

// This will run when the test binary exits
impl Drop for GlobalTestApp {
    fn drop(&mut self) {
        eprintln!("🧹 Cleaning up global test environment...");

        // Kill the catalog service process
        if let Some(mut process) = self.catalog_process.take() {
            process.kill().ok();
            process.wait().ok();
        }

        // Schedule database cleanup
        let client = self.mongodb_client.clone();
        let db_name = self.db_name.clone();
        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                if let Err(e) = client.database(&db_name).drop().await {
                    eprintln!("Failed to drop test database {db_name}: {e}");
                } else {
                    eprintln!("✅ Dropped test database: {db_name}");
                }
            });
        });

        eprintln!("✅ Global test environment cleaned up");
    }
}
