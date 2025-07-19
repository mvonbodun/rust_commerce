// Include only the model module for shared use
#[path = "catalog-service/model.rs"]
pub mod model;

// Re-export the models at the crate level for easier importing
pub use model::*;
