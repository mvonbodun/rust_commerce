use std::process::{Child, Command};
use std::time::Duration;
use tokio::time::sleep;

/// Manages a catalog service instance for testing
pub struct TestCatalogService {
    process: Option<Child>,
    db_name: String,
}

impl TestCatalogService {
    /// Start a catalog service with a test database
    pub async fn start(db_name: String) -> Self {
        // Kill any existing catalog service
        Command::new("pkill")
            .args(["-f", "catalog-service"])
            .output()
            .ok();

        // Wait a moment for the process to die
        sleep(Duration::from_millis(500)).await;

        // Start the catalog service with test database
        let process = Command::new("cargo")
            .args(["run", "--bin", "catalog-service"])
            .env("CATALOG_DB_NAME", &db_name)
            .env("RUST_LOG", "info")
            .spawn()
            .expect("Failed to start catalog service");

        // Wait for service to be ready
        sleep(Duration::from_secs(3)).await;

        Self {
            process: Some(process),
            db_name,
        }
    }

    pub fn db_name(&self) -> &str {
        &self.db_name
    }
}

impl Drop for TestCatalogService {
    fn drop(&mut self) {
        if let Some(mut process) = self.process.take() {
            process.kill().ok();
            process.wait().ok();
        }
    }
}
