use crate::helpers::test_service::TestCatalogService;
use mongodb::Client;
use rust_common::test_helpers::TestApp;
use std::sync::Once;

static INIT: Once = Once::new();
static mut SERVICE: Option<TestCatalogService> = None;

/// Initialize test infrastructure once per test run
pub async fn init_test_infrastructure() {
    INIT.call_once(|| {
        // This runs only once for all tests
        // Logger is already initialized by the service
    });
}

/// Create a test app with isolated database
pub async fn create_test_app() -> (TestApp, String) {
    init_test_infrastructure().await;

    // Generate unique test database name
    let test_db_name = format!(
        "test_catalog_{}",
        uuid::Uuid::new_v4().to_string().replace("-", "")
    );

    // Start catalog service with test database
    unsafe {
        if SERVICE.is_none() {
            SERVICE = Some(TestCatalogService::start(test_db_name.clone()).await);
        } else {
            // Kill existing service and start new one with test database
            drop(SERVICE.take());
            SERVICE = Some(TestCatalogService::start(test_db_name.clone()).await);
        }
    }

    // Create test app
    let app = TestApp::spawn().await;

    (app, test_db_name)
}

/// Clean up test database after test
pub async fn cleanup_test_db(db_name: &str) {
    if let Ok(client) = Client::with_uri_str("mongodb://admin:changeme@localhost:27017").await {
        let _ = client.database(db_name).drop().await;
    }
}

/// Test context that handles setup and cleanup
pub struct TestContext {
    pub app: TestApp,
    pub db_name: String,
}

impl TestContext {
    pub async fn new() -> Self {
        let (app, db_name) = create_test_app().await;
        Self { app, db_name }
    }
}

impl Drop for TestContext {
    fn drop(&mut self) {
        // Schedule cleanup of test database
        let db_name = self.db_name.clone();
        tokio::spawn(async move {
            cleanup_test_db(&db_name).await;
        });
    }
}
