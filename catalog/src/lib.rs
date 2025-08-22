// Include the modules for shared use
#[path = "catalog-service/domain/mod.rs"]
pub mod domain;

#[path = "catalog-service/handlers/mod.rs"]
pub mod handlers;

#[path = "catalog-service/persistence/mod.rs"]
pub mod persistence;

#[path = "catalog-service/services/mod.rs"]
pub mod services;

#[path = "catalog-service/startup.rs"]
pub mod startup;

// AppState definition
#[derive(Clone)]
pub struct AppState {
    pub product_dao: std::sync::Arc<dyn persistence::product_dao::ProductDao + Send + Sync>,
    pub category_dao: std::sync::Arc<dyn persistence::category_dao::CategoryDao + Send + Sync>,
    pub product_service: std::sync::Arc<services::product_service::ProductService>,
    pub category_service: std::sync::Arc<services::category_service::CategoryService>,
}

// Import common module for generated proto code
mod common {
    pub use shared_proto::common::*;
}

pub mod catalog_messages {
    include!(concat!(env!("OUT_DIR"), "/catalog_messages.rs"));

    // Re-export common types for backward compatibility
    pub use super::common::{Code, Status};
}

// Re-export the domain types at the crate level for easier importing
pub use domain::*;
