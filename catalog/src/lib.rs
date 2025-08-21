// Include the domain module for shared use
#[path = "catalog-service/domain/mod.rs"]
pub mod domain;

// Re-export the domain types at the crate level for easier importing
pub use domain::*;
