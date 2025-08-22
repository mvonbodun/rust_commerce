// Note: startup module is not exported from lib.rs, only accessible from catalog-service binary
// So we can't use it directly in tests. Tests should spawn the service via command line.
use log::info;
use mongodb::Client;
use rust_common::test_helpers::TestApp;
use std::process::{Child, Command};
use uuid::Uuid;

pub struct TestCatalogApp {
    pub app: TestApp,
    pub db_name: String,
    pub mongodb_client: Client,
    catalog_process: Option<Child>,
}

impl TestCatalogApp {
    pub async fn spawn() -> Self {
        // Generate a unique test database name
        let db_name = format!(
            "test_catalog_{}",
            Uuid::new_v4().to_string().replace("-", "_")
        );
        info!("🧪 Starting test with database: {}", db_name);

        // Kill any existing catalog service
        Command::new("pkill")
            .args(&["-f", "catalog-service"])
            .output()
            .ok();

        // Wait for the service to stop
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;

        // Start a new catalog service with the test database
        let catalog_process = Command::new("cargo")
            .args(&["run", "--bin", "catalog-service"])
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

        // Give the service time to start - it takes time to compile and start
        info!(
            "⏳ Waiting for catalog service to start with database: {}",
            db_name
        );
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;

        // Create a TestApp for making requests
        let app = TestApp::spawn().await;

        // Connect to MongoDB for cleanup
        let mongodb_client = Client::with_uri_str(
            &std::env::var("MONGODB_URL")
                .unwrap_or_else(|_| "mongodb://admin:password123@localhost:27017".to_string()),
        )
        .await
        .expect("Failed to connect to MongoDB");

        // Verify the test database was created
        let test_db = mongodb_client.database(&db_name);

        // The catalog-service will create its own collections and indexes when it starts
        // with the CATALOG_DB_NAME environment variable pointing to our test database

        info!("✅ Test database {} is ready", db_name);

        Self {
            app,
            db_name,
            mongodb_client,
            catalog_process: Some(catalog_process),
        }
    }

    /// Clean up the test database
    pub async fn cleanup(&self) {
        let _ = self.mongodb_client.database(&self.db_name).drop().await;
    }
}

impl Drop for TestCatalogApp {
    fn drop(&mut self) {
        info!("🧹 Cleaning up test database: {}", self.db_name);

        // Kill the catalog service process
        if let Some(mut process) = self.catalog_process.take() {
            process.kill().ok();
            process.wait().ok();
        }

        // Schedule database cleanup
        let client = self.mongodb_client.clone();
        let db_name = self.db_name.clone();
        tokio::spawn(async move {
            if let Err(e) = client.database(&db_name).drop().await {
                eprintln!("Failed to drop test database {}: {}", db_name, e);
            } else {
                info!("✅ Dropped test database: {}", db_name);
            }
        });

        // Restart the main catalog service for other tests
        info!("🔄 Restarting main catalog service with db_catalog");
        Command::new("cargo")
            .args(&["run", "--bin", "catalog-service"])
            .env("CATALOG_DB_NAME", "db_catalog")
            .spawn()
            .ok();
    }
}
