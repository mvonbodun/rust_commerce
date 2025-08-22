// Note: startup module is not exported from lib.rs, only accessible from catalog-service binary
// So we can't use it directly in tests. Tests should spawn the service via command line.
use log::{debug, info};
use rust_catalog::startup::{Application, Settings};
use rust_common::{load_environment, mask_sensitive_url, test_helpers::{TestApp, TestConfig}};
// use std::process::{Command};
use uuid::Uuid;

// pub struct TestApp {
//     pub nats_url: String,
//     pub nats_client: async_nats::Client,
//     pub db_name: String,
// }

// impl TestApp {
//     /// Execute a NATS request and return the response
//     pub async fn request(
//         &self,
//         subject: &str,
//         payload: Vec<u8>,
//     ) -> Result<async_nats::Message, Box<dyn std::error::Error + Send + Sync>> {
//         Ok(self
//             .nats_client
//             .request(subject.to_string(), payload.into())
//             .await?)
//     }
// }

pub async fn spawn_app() -> TestApp {
    // Generate a unique test database name
    let db_name = format!(
        "test_catalog_{}",
        Uuid::new_v4().to_string().replace("-", "_")
    );
    info!("🧪 Starting test with database: {db_name}");

    // Kill any existing catalog service
    // debug!("🔪 Killing any existing catalog service processes");
    // Command::new("pkill")
    //     .args(["-f", "catalog-service"])
    //     .output()
    //     .ok();

    // Wait for the service to stop
    debug!("⏳ Waiting for catalog service to stop");
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    // Start a new catalog service with the test database
    // Assumes that NATS server is already running

    // Load enviroment variables
    load_environment();

    // Initialize logger after loading environment (so RUST_LOG from .env is used)
    // pretty_env_logger::init();

    // Create the Settings
    let mut settings: Settings = Settings::from_env();
    let mongodb_url = settings.mongodb_url.clone();
    // Use the specialized db_name generated for testing
    settings.database_name = db_name.clone();
    info!(
        "  MONGODB_URL: {}",
        mask_sensitive_url(&settings.mongodb_url)
    );
    info!("  NATS_URL: {}", settings.nats_url);
    info!("  CATALOG_DB_NAME: {}", settings.database_name);

    // Save the NATS URL before moving settings
    let nats_url = settings.nats_url.clone();

    // Build the application
    let app = Application::build(settings)
        .await
        .expect("Failed to build application");

    // Run the application in the background
    let _ = tokio::spawn(async move {
        if let Err(e) = app.run().await {
            eprintln!("Application error: {e}");
        }
    });

    // Give the service time to start
    // info!("⏳ Waiting for catalog service to start with database: {db_name}");
    // tokio::time::sleep(std::time::Duration::from_secs(2)).await;

    // let catalog_process = Command::new("cargo")
    //     .args(["run", "--bin", "catalog-service"])
    //     .env("CATALOG_DB_NAME", &db_name)
    //     .env(
    //         "MONGODB_URL",
    //         std::env::var("MONGODB_URL")
    //             .unwrap_or_else(|_| "mongodb://admin:password123@localhost:27017".to_string()),
    //     )
    //     .env(
    //         "NATS_URL",
    //         std::env::var("NATS_URL").unwrap_or_else(|_| "nats://localhost:4222".to_string()),
    //     )
    //     .env("RUST_LOG", "info")
    //     .spawn()
    //     .expect("Failed to start catalog service");

    // Give the service time to start - it takes time to compile and start
    // info!("⏳ Waiting for catalog service to start with database: {db_name}");
    // tokio::time::sleep(std::time::Duration::from_secs(5)).await;

    // Create a NATS client for the test app
    // let nats_client = async_nats::connect(&nats_url)
    //     .await
    //     .expect("Failed to connect to NATS");

    // Create a TestApp for making requests
    let test_app = TestApp::new_with_config(TestConfig {
        nats_url,
        mongodb_url: mongodb_url.clone(),
        test_db_name: db_name.clone(),
    })
    .await;

    // // Connect to MongoDB for cleanup
    // let mongodb_client = Client::with_uri_str(
    //     &std::env::var("MONGODB_URL")
    //         .unwrap_or_else(|_| "mongodb://admin:password123@localhost:27017".to_string()),
    // )
    // .await
    // .expect("Failed to connect to MongoDB");

    // // Verify the test database was created
    // let test_db = mongodb_client.database(&db_name);

    // The catalog-service will create its own collections and indexes when it starts
    // with the CATALOG_DB_NAME environment variable pointing to our test database

    info!("✅ Test catalog service started");
    test_app
}

// /// Cleanup on drop
// impl Drop for TestApp {
//     fn drop(&mut self) {
//         let client = self.mongodb_client.clone();
//         let db_name = self.test_db_name.clone();

//         // Spawn a blocking task to clean up the database
//         // We can't use async in Drop, so we spawn a task
//         std::thread::spawn(move || {
//             let rt = tokio::runtime::Runtime::new().unwrap();
//             rt.block_on(async {
//                 let _ = cleanup_test_db(&client, &db_name).await;
//             });
//         });
//     }
// }

// /// Clean up the test database
// pub async fn cleanup(&self) {
//     let _ = self.mongodb_client.database(&self.db_name).drop().await;
// }

// impl Drop for TestCatalogApp {
//     fn drop(&mut self) {
//         info!("🧹 Cleaning up test database: {}", self.db_name);

//         // Kill the catalog service process
//         if let Some(mut process) = self.catalog_process.take() {
//             process.kill().ok();
//             process.wait().ok();
//         }

//         // Schedule database cleanup
//         let client = self.mongodb_client.clone();
//         let db_name = self.db_name.clone();
//         tokio::spawn(async move {
//             if let Err(e) = client.database(&db_name).drop().await {
//                 eprintln!("Failed to drop test database {db_name}: {e}");
//             } else {
//                 info!("✅ Dropped test database: {db_name}");
//             }
//         });

//         // Restart the main catalog service for other tests
//         info!("🔄 Restarting main catalog service with db_catalog");
//         Command::new("cargo")
//             .args(["run", "--bin", "catalog-service"])
//             .env("CATALOG_DB_NAME", "db_catalog")
//             .spawn()
//             .ok();
//     }
// }
