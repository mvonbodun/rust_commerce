// Library exports for the price service

// Include generated protobuf code
pub mod offer_messages {
    include!(concat!(env!("OUT_DIR"), "/offer_messages.rs"));
}

// Model types
#[path = "price-service/model.rs"]
pub mod model;

pub mod env_config;

// Persistence layer
#[path = "price-service/persistence"]
pub mod persistence {
    pub mod offer_dao;
}

// Handlers for gRPC/NATS
#[path = "price-service/handlers"]
pub mod handlers {
    pub mod handlers_inner;
    
    #[path = "mod.rs"]
    pub mod handler_mod;
}

// Re-export commonly used types
pub use model::{Offer, OfferPrice, DBError};
pub use persistence::offer_dao::{OfferDao, OfferDaoImpl};
