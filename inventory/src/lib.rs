// Library exports for the inventory service

// Include generated protobuf code
pub mod inventory_messages {
    include!(concat!(env!("OUT_DIR"), "/inventory_messages.rs"));
}

// Model types
#[path = "inventory-service/model.rs"]
pub mod model;

// Persistence layer
#[path = "inventory-service/persistence"]
pub mod persistence {
    pub mod inventory_dao;
}

// Handlers for gRPC/NATS
#[path = "inventory-service/handlers/mod.rs"]
pub mod handlers;
pub use handlers::handlers_inner;
