use async_nats::{Client, Message};
use std::collections::HashMap;

use crate::AppState;
pub mod category_handlers;
pub mod product_handlers;

pub type RouteHandler = Box<
    dyn Fn(
            std::sync::Arc<AppState>,
            Client,
            Message,
        ) -> std::pin::Pin<
            Box<
                dyn std::future::Future<
                        Output = Result<(), Box<dyn std::error::Error + Send + Sync>>,
                    > + Send,
            >,
        > + Send
        + Sync,
>;

pub struct Router {
    pub route_map: HashMap<String, RouteHandler>,
}

impl Default for Router {
    fn default() -> Self {
        Self::new()
    }
}

impl Router {
    pub fn new() -> Self {
        Self {
            route_map: HashMap::new(),
        }
    }

    pub fn add_route(&mut self, path: String, handler: RouteHandler) -> &mut Self {
        self.route_map.insert(path, handler);
        self
    }
}
