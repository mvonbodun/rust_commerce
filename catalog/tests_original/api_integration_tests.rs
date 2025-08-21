// Integration tests for the Catalog service API
// These tests exercise the full client-service communication via NATS

#[path = "../tests_api/mod.rs"]
mod api;

// Re-export test modules
use api::commands::product_tests;
use api::commands::category_tests;