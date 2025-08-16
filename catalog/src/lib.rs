// Include only the model module for shared use
#[path = "catalog-service/model.rs"]
pub mod model;

// Re-export the models at the crate level for easier importing
pub use model::*;

pub mod env_config;
pub mod logging_utils;
