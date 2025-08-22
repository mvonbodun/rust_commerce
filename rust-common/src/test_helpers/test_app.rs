use async_nats::Client;
use log::info;
use mongodb::Database;

use super::{cleanup_test_db, TestConfig};

/// TestApp manages the lifecycle of a test environment
pub struct TestApp {
    pub nats_client: Client,
    pub mongodb_client: mongodb::Client,
    pub mongodb_db: Database,
    pub test_db_name: String,
    #[allow(dead_code)]
    pub config: TestConfig,
}

impl TestApp {
    // Create a new test application instance
    pub async fn new() -> Self {
        Self::new_with_config(TestConfig::default()).await
    }

    /// Create a new test application instance with the given configuration
    pub async fn new_with_config(config: TestConfig) -> Self {
        // Connect to NATS
        let nats_client = async_nats::connect(&config.nats_url)
            .await
            .expect("Failed to connect to NATS for testing");

        // Connect to MongoDB
        let mongodb_client = mongodb::Client::with_uri_str(&config.mongodb_url)
            .await
            .expect("Failed to connect to MongoDB for testing");

        // Create unique test database
        info!("🧪 Using test database: {}", &config.test_db_name);
        let mongodb_db = mongodb_client.database(&config.test_db_name);

        Self {
            nats_client,
            mongodb_client,
            mongodb_db,
            test_db_name: config.test_db_name.clone(),
            config,
        }
    }

    /// Get a NATS client for making requests
    pub fn nats(&self) -> &Client {
        &self.nats_client
    }

    /// Get the test database
    pub fn db(&self) -> &Database {
        &self.mongodb_db
    }

    /// Execute a NATS request and return the response
    pub async fn request(
        &self,
        subject: &str,
        payload: Vec<u8>,
    ) -> Result<async_nats::Message, Box<dyn std::error::Error + Send + Sync>> {
        Ok(self
            .nats_client
            .request(subject.to_string(), payload.into())
            .await?)
    }

    /// Execute a NATS request with timeout
    pub async fn request_with_timeout(
        &self,
        subject: &str,
        payload: Vec<u8>,
        timeout: std::time::Duration,
    ) -> Result<async_nats::Message, Box<dyn std::error::Error + Send + Sync>> {
        tokio::time::timeout(timeout, self.request(subject, payload))
            .await
            .map_err(|_| {
                Box::new(std::io::Error::new(
                    std::io::ErrorKind::TimedOut,
                    "Request timed out",
                )) as Box<dyn std::error::Error + Send + Sync>
            })?
    }

    /// Insert test data directly into MongoDB
    pub async fn insert_test_data<T>(
        &self,
        collection: &str,
        data: T,
    ) -> Result<mongodb::results::InsertOneResult, mongodb::error::Error>
    where
        T: serde::Serialize,
    {
        let doc = bson::to_document(&data).unwrap();
        self.mongodb_db
            .collection::<bson::Document>(collection)
            .insert_one(doc)
            .await
    }

    /// Count documents in a collection
    pub async fn count_documents(&self, collection: &str) -> Result<u64, mongodb::error::Error> {
        self.mongodb_db
            .collection::<bson::Document>(collection)
            .count_documents(bson::doc! {})
            .await
    }

    /// Clear a collection
    pub async fn clear_collection(
        &self,
        collection: &str,
    ) -> Result<mongodb::results::DeleteResult, mongodb::error::Error> {
        self.mongodb_db
            .collection::<bson::Document>(collection)
            .delete_many(bson::doc! {})
            .await
    }

    /// Explicitly cleanup the test database (useful if Drop isn't working)
    pub async fn cleanup(&self) -> Result<(), mongodb::error::Error> {
        eprintln!("🧹 Explicitly cleaning up test database: {}", self.test_db_name);
        cleanup_test_db(&self.mongodb_client, &self.test_db_name).await
    }
}

/// Cleanup on drop
impl Drop for TestApp {
    fn drop(&mut self) {
        let client = self.mongodb_client.clone();
        let db_name = self.test_db_name.clone();

        // Spawn a blocking task to clean up the database
        // We can't use async in Drop, so we spawn a task
        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let _ = cleanup_test_db(&client, &db_name).await;
            });
        });
    }
}
