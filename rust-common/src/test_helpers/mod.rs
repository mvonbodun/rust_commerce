// Shared test infrastructure for integration testing
pub mod fixtures;
pub mod test_app;

pub use fixtures::*;
pub use test_app::*;

use uuid::Uuid;

/// Configuration for test environment
pub struct TestConfig {
    pub nats_url: String,
    pub mongodb_url: String,
    pub test_db_prefix: String,
}

impl Default for TestConfig {
    fn default() -> Self {
        Self {
            nats_url: std::env::var("NATS_TEST_URL")
                .unwrap_or_else(|_| "nats://127.0.0.1:4222".to_string()),
            mongodb_url: std::env::var("MONGODB_TEST_URL").unwrap_or_else(|_| {
                "mongodb://admin:password123@localhost:27017/?authSource=admin".to_string()
            }),
            test_db_prefix: "test_".to_string(),
        }
    }
}

/// Generate a unique test database name
pub fn generate_test_db_name() -> String {
    format!("test_{}", Uuid::new_v4().to_string().replace("-", ""))
}

/// Clean up test database after test completion
pub async fn cleanup_test_db(
    client: &mongodb::Client,
    db_name: &str,
) -> Result<(), mongodb::error::Error> {
    client.database(db_name).drop().await
}

/// Helper trait for response assertions
pub trait ResponseAssertions {
    fn assert_ok(&self);
    fn assert_error(&self, expected_code: i32);
    fn assert_contains(&self, expected: &str);
}

/// Test data validation helpers
pub mod validators {
    /// Check if a string contains SQL injection attempts
    pub fn contains_sql_injection(input: &str) -> bool {
        let patterns = vec![
            "DROP TABLE",
            "DELETE FROM",
            "INSERT INTO",
            "UPDATE SET",
            "OR 1=1",
            "'; --",
            "\" OR \"",
        ];

        let input_upper = input.to_uppercase();
        patterns.iter().any(|pattern| input_upper.contains(pattern))
    }

    /// Check if a string contains XSS attempts
    pub fn contains_xss(input: &str) -> bool {
        let patterns = vec![
            "<script",
            "</script>",
            "javascript:",
            "onclick=",
            "onerror=",
            "<iframe",
            "alert(",
        ];

        let input_lower = input.to_lowercase();
        patterns.iter().any(|pattern| input_lower.contains(pattern))
    }

    /// Check if a string is within valid length bounds
    pub fn is_valid_length(input: &str, min: usize, max: usize) -> bool {
        let len = input.len();
        len >= min && len <= max
    }
}

/// Common test assertions
pub mod assertions {
    use prost::Message;
    use uuid::Uuid;

    pub fn assert_status_ok<T: Message>(_response: &T, status_field: Option<i32>) {
        assert_eq!(status_field, Some(0), "Expected OK status (0)");
    }

    pub fn assert_status_error<T: Message>(
        _response: &T,
        status_field: Option<i32>,
        expected_code: i32,
    ) {
        assert_eq!(
            status_field,
            Some(expected_code),
            "Expected error code {}",
            expected_code
        );
    }

    pub fn assert_not_empty(value: &str, field_name: &str) {
        assert!(!value.is_empty(), "{} should not be empty", field_name);
    }

    pub fn assert_uuid_format(value: &str, field_name: &str) {
        assert!(
            Uuid::parse_str(value).is_ok(),
            "{} should be a valid UUID, got: {}",
            field_name,
            value
        );
    }
}

/// Performance measurement utilities
pub mod performance {
    use std::time::{Duration, Instant};

    pub struct Timer {
        start: Instant,
        name: String,
    }

    impl Timer {
        pub fn start(name: impl Into<String>) -> Self {
            Self {
                start: Instant::now(),
                name: name.into(),
            }
        }

        pub fn elapsed(&self) -> Duration {
            self.start.elapsed()
        }

        pub fn assert_under(&self, max_duration: Duration) {
            let elapsed = self.elapsed();
            assert!(
                elapsed < max_duration,
                "{} took {:?}, expected under {:?}",
                self.name,
                elapsed,
                max_duration
            );
        }
    }

    impl Drop for Timer {
        fn drop(&mut self) {
            println!("{} completed in {:?}", self.name, self.elapsed());
        }
    }
}

/// Retry logic for flaky operations
pub async fn retry_async<F, Fut, T, E>(
    mut f: F,
    max_attempts: u32,
    delay: std::time::Duration,
) -> Result<T, E>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T, E>>,
{
    let mut attempt = 1;
    loop {
        match f().await {
            Ok(result) => return Ok(result),
            Err(e) if attempt >= max_attempts => return Err(e),
            Err(_) => {
                tokio::time::sleep(delay).await;
                attempt += 1;
            }
        }
    }
}
