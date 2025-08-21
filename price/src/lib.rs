// Library exports for the price service

// Import common module for generated proto code
mod common {
    pub use shared_proto::common::*;
}

// Include generated protobuf code
pub mod offer_messages {
    include!(concat!(env!("OUT_DIR"), "/offer_messages.rs"));

    // Re-export common types for backward compatibility
    pub use super::common::{Code, Status};
}

// Model types
#[path = "price-service/model.rs"]
pub mod model;

// Persistence layer
#[path = "price-service/persistence"]
pub mod persistence {
    pub mod offer_dao;
}

// Handlers for gRPC/NATS
#[path = "price-service/handlers/mod.rs"]
pub mod handlers;
pub use handlers::handlers_inner;

// Re-export commonly used types
pub use model::{DBError, Offer, OfferPrice};
pub use persistence::offer_dao::{OfferDao, OfferDaoImpl};
