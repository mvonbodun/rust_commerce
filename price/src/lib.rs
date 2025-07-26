// Re-export model types from the price service
pub use crate::price_service::model::{Offer, OfferPrice};

// Module declarations
#[path = "price-service/model.rs"]
pub mod model;

// Create a module alias for convenience
pub mod price_service {
    pub use super::model;
}
