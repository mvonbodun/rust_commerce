# Logging & Tracing Improvement Plan

## Overview
This plan outlines comprehensive improvements to logging and tracing across the Rust Commerce microservices, with a focus on enhanced visibility during startup, runtime operations, and error handling.

## Current State Analysis
- ✅ Basic `pretty_env_logger::init()` setup
- ✅ Minimal startup logging
- ✅ MongoDB connection logging
- ✅ NATS connection logging
- ✅ Database/collection setup logging
- ✅ Index creation progress logging
- ✅ Service initialization logging
- ✅ Error context in connection failures
- ✅ Layered environment configuration (.env → .env.local → system env)
- ✅ URL masking for security
- ✅ Request processing timing

## Implementation Phases

### Phase 1: Enhanced Startup Logging ✅ **COMPLETED**

#### 1.1 Environment & Configuration Logging
```rust
info!("Starting Rust Commerce Catalog Service v{}", env!("CARGO_PKG_VERSION"));
info!("Environment configuration:");
info!("  RUST_ENV: {}", env::var("RUST_ENV").unwrap_or_else(|_| "development".to_string()));
info!("  RUST_LOG: {}", env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string()));
info!("  MONGODB_URL: {}", mask_sensitive_url(&uri));
info!("  NATS_URL: {}", &nats_url);
```

#### 1.2 MongoDB Connection Logging
```rust
info!("🔗 Connecting to MongoDB...");
let client = match Client::with_uri_str(&uri).await {
    Ok(client) => {
        info!("✅ Successfully connected to MongoDB");
        // Test the connection
        match client.list_database_names().await {
            Ok(databases) => {
                info!("📋 Available databases: {:?}", databases);
                client
            }
            Err(e) => {
                error!("❌ Failed to list databases: {}", e);
                return Err(e.into());
            }
        }
    }
    Err(e) => {
        error!("❌ Failed to connect to MongoDB: {}", e);
        return Err(e.into());
    }
};
```

#### 1.3 Collection & Index Setup Logging
```rust
info!("📦 Setting up products collection...");
info!("🔍 Creating product indexes...");
match products_coll.create_indexes(indexes).await {
    Ok(result) => info!("✅ Created {} product indexes", result.len()),
    Err(e) => {
        error!("❌ Failed to create product indexes: {}", e);
        return Err(e.into());
    }
}
```

#### 1.4 NATS Connection Logging
```rust
info!("🔗 Connecting to NATS server: {}", nats_url);
let nats_client = match async_nats::connect(&nats_url).await {
    Ok(client) => {
        info!("✅ Successfully connected to NATS");
        client
    }
    Err(e) => {
        error!("❌ Failed to connect to NATS: {}", e);
        return Err(e.into());
    }
};
```

### Phase 2: Service Initialization Logging ✅ **COMPLETED**

#### 2.1 DAO & Service Setup
```rust
info!("🏗️  Initializing data access objects...");
let product_dao = Arc::new(ProductDaoImpl::new(products_coll, database.clone()));
debug!("✅ Product DAO initialized");

let category_dao = Arc::new(CategoryDaoImpl::new(categories_coll, category_cache_coll));
debug!("✅ Category DAO initialized");

let category_service = Arc::new(CategoryService::new(category_dao));
debug!("✅ Category Service initialized");
```

#### 2.2 Router Setup Logging
```rust
info!("🛣️  Setting up message router...");
let mut router = Router::new();
// ... add routes ...
info!("✅ Configured {} routes for catalog operations", route_count);
debug!("Routes: create_product, get_product, get_product_by_slug, update_product, delete_product, search_products, export_products, get_product_slugs");
```

### Phase 3: Runtime Logging ✅ **COMPLETED**

#### 3.1 Queue Subscription Logging
```rust
info!("📡 Subscribing to NATS queue: catalog.*");
let requests = match nats_client.queue_subscribe("catalog.*", "queue".to_owned()).await {
    Ok(subscription) => {
        info!("✅ Successfully subscribed to catalog.* queue");
        subscription
    }
    Err(e) => {
        error!("❌ Failed to subscribe to NATS queue: {}", e);
        return Err(e.into());
    }
};

info!("🚀 Catalog service is ready and listening for requests");
info!("📊 Service startup completed successfully");
```

#### 3.2 Request Processing Logging
```rust
async move {
    let operation = subject_parts[1].to_string();
    debug!("📨 Processing catalog operation: {} from subject: {}", operation, request.subject);
    
    let start_time = std::time::Instant::now();
    
    // ... processing logic ...
    
    match result {
        Ok(_) => {
            let elapsed = start_time.elapsed();
            debug!("✅ Successfully processed {} in {:?}", operation, elapsed);
        }
        Err(e) => {
            let elapsed = start_time.elapsed();
            error!("❌ Error processing {} after {:?}: {:?}", operation, elapsed, e);
        }
    }
}
```

### Phase 4: Error Context & Health Monitoring ⭐ **IMPLEMENTING**

#### 4.1 Connection Health Checks
```rust
// Add periodic health checks for critical connections
async fn health_check_task(client: mongodb::Client, nats_client: async_nats::Client) {
    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(30));
    loop {
        interval.tick().await;
        
        // MongoDB health check
        match client.list_database_names().await {
            Ok(_) => debug!("💓 MongoDB health check: OK"),
            Err(e) => error!("💔 MongoDB health check failed: {}", e),
        }
        
        // NATS health check  
        match nats_client.connection_state() {
            async_nats::connection::State::Connected => debug!("💓 NATS health check: OK"),
            state => warn!("💔 NATS connection state: {:?}", state),
        }
    }
}
```

#### 4.2 Graceful Shutdown Logging
```rust
// Add signal handling for graceful shutdown
async fn setup_signal_handlers() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use tokio::signal;
    
    tokio::spawn(async {
        let mut sigterm = signal::unix::signal(signal::unix::SignalKind::terminate()).unwrap();
        let mut sigint = signal::unix::signal(signal::unix::SignalKind::interrupt()).unwrap();
        
        tokio::select! {
            _ = sigterm.recv() => {
                info!("🛑 Received SIGTERM, initiating graceful shutdown...");
            }
            _ = sigint.recv() => {
                info!("🛑 Received SIGINT, initiating graceful shutdown...");
            }
        }
        
        info!("🧹 Cleaning up resources...");
        // Add cleanup logic here
        info!("✅ Graceful shutdown completed");
        std::process::exit(0);
    });
    
    Ok(())
}
```

#### 4.3 Service Dependency Validation
```rust
// Add startup dependency validation
async fn validate_dependencies(
    mongo_client: &mongodb::Client,
    nats_client: &async_nats::Client,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    info!("🔍 Validating service dependencies...");
    
    // Validate MongoDB collections exist
    let database = mongo_client.database("db_catalog");
    let collections = database.list_collection_names().await?;
    
    for required_collection in &["products", "categories", "category_tree_cache"] {
        if collections.contains(&required_collection.to_string()) {
            debug!("✅ Collection '{}' exists", required_collection);
        } else {
            warn!("⚠️  Collection '{}' not found, will be created on first use", required_collection);
        }
    }
    
    // Validate NATS subjects can be subscribed to
    match nats_client.queue_subscribe("catalog.health_check", "test_queue".to_string()).await {
        Ok(_) => {
            debug!("✅ NATS subscription test successful");
        }
        Err(e) => {
            error!("❌ NATS subscription test failed: {}", e);
            return Err(e.into());
        }
    }
    
    info!("✅ All dependencies validated successfully");
    Ok(())
}
```

### Phase 5: Utility Functions ✅ **COMPLETED**

#### 5.1 URL Masking for Security ✅
```rust
// Already implemented in catalog service
fn mask_sensitive_url(url: &str) -> String {
    // Simple pattern matching to mask passwords in MongoDB URLs
    if url.contains("://") && url.contains("@") {
        let parts: Vec<&str> = url.split("://").collect();
        if parts.len() == 2 {
            let scheme = parts[0];
            let rest = parts[1];
            
            if let Some(at_pos) = rest.find('@') {
                let auth_part = &rest[..at_pos];
                let host_part = &rest[at_pos..];
                
                // Mask password if present
                if let Some(colon_pos) = auth_part.find(':') {
                    let username = &auth_part[..colon_pos];
                    return format!("{}://{}:***{}", scheme, username, host_part);
                }
            }
        }
    }
    url.to_string()
}
```

#### 5.2 Performance Timing Utilities
```rust
// Add timing utilities for performance monitoring
pub struct OperationTimer {
    start: std::time::Instant,
    operation: String,
}

impl OperationTimer {
    pub fn new(operation: &str) -> Self {
        debug!("⏱️  Starting operation: {}", operation);
        Self {
            start: std::time::Instant::now(),
            operation: operation.to_string(),
        }
    }
    
    pub fn log_elapsed(&self, level: &str) {
        let elapsed = self.start.elapsed();
        match level {
            "debug" => debug!("⏱️  {} completed in {:?}", self.operation, elapsed),
            "info" => info!("⏱️  {} completed in {:?}", self.operation, elapsed),
            "warn" => warn!("⏱️  {} took {:?} (slower than expected)", self.operation, elapsed),
            _ => debug!("⏱️  {} completed in {:?}", self.operation, elapsed),
        }
    }
}

impl Drop for OperationTimer {
    fn drop(&mut self) {
        self.log_elapsed("debug");
    }
}
```

#### 5.3 Error Context Enhancement
```rust
// Enhanced error context for better debugging
pub trait ErrorContext<T> {
    fn with_context(self, context: &str) -> Result<T, Box<dyn std::error::Error + Send + Sync>>;
}

impl<T, E> ErrorContext<T> for Result<T, E>
where
    E: std::error::Error + Send + Sync + 'static,
{
    fn with_context(self, context: &str) -> Result<T, Box<dyn std::error::Error + Send + Sync>> {
        self.map_err(|e| {
            error!("❌ {}: {}", context, e);
            Box::new(e) as Box<dyn std::error::Error + Send + Sync>
        })
    }
}
```

### Phase 6: Log Level Recommendations ⭐ **IMPLEMENTING**

#### 6.1 Environment-Specific Logging Configuration
```bash
# Development (.env.local)
RUST_LOG=debug,catalog_service=trace,mongodb=debug,async_nats=debug

# Staging (.env.staging)  
RUST_LOG=info,catalog_service=debug,mongodb=warn,async_nats=warn

# Production (.env.production)
RUST_LOG=info,mongodb=error,async_nats=error,h2=error,hyper=error

# Troubleshooting (temporary override)
RUST_LOG=trace,catalog_service=trace,mongodb=debug,async_nats=debug
```

#### 6.2 Log Level Documentation
```rust
// Add log level guidance as comments in code
//
// Log Level Guidelines:
// - ERROR: Critical failures that require immediate attention
// - WARN:  Unexpected conditions that don't stop operation
// - INFO:  Important operational events (startup, shutdown, major operations)
// - DEBUG: Detailed flow information for development/troubleshooting
// - TRACE: Very detailed information, including data dumps
//
// Service-specific recommendations:
// - catalog_service: Use DEBUG for development, INFO for production
// - mongodb: Use WARN to avoid noise, DEBUG only when investigating DB issues
// - async_nats: Use WARN to avoid connection noise, DEBUG for message tracing
// - hyper/h2: Use ERROR only, these are very noisy in debug mode
```

#### 6.3 Dynamic Log Level Control
```rust
// Add runtime log level adjustment capability
pub fn adjust_log_level(target: &str, level: &str) {
    use log::LevelFilter;
    
    let filter = match level.to_lowercase().as_str() {
        "error" => LevelFilter::Error,
        "warn" => LevelFilter::Warn, 
        "info" => LevelFilter::Info,
        "debug" => LevelFilter::Debug,
        "trace" => LevelFilter::Trace,
        _ => {
            warn!("⚠️  Invalid log level '{}', keeping current level", level);
            return;
        }
    };
    
    info!("🔧 Adjusting log level for '{}' to '{}'", target, level);
    // Implementation would require env_logger rebuild or tracing subscriber
}
```

#### 6.4 Log Rotation and Management
```bash
# Production log management recommendations

# For Docker environments, use structured logging with JSON output:
RUST_LOG=info
RUST_LOG_FORMAT=json

# Log rotation with logrotate (production systems):
# /etc/logrotate.d/rust-commerce
/var/log/rust-commerce/*.log {
    daily
    rotate 30
    compress
    delaycompress
    missingok
    notifempty
    create 644 app app
    postrotate
        /usr/bin/systemctl reload rust-commerce
    endscript
}
```

### Phase 7: Structured Logging Migration (Future)

Consider migrating from `env_logger` to `tracing` for better structured logging with spans and events.

## Benefits

### Immediate Benefits (Phases 1-3)
- **🔍 Better Debugging**: Clear visibility into startup failures
- **📊 Operational Insights**: Understanding service initialization flow
- **⚡ Performance Monitoring**: Request processing timing
- **🛡️ Security**: Masked sensitive information in logs

### Future Benefits (Phases 4-7)
- **💓 Health Monitoring**: Proactive issue detection
- **🏗️ Structured Data**: Better log analysis and alerting
- **📈 Observability**: Enhanced production monitoring
- **🔧 Maintenance**: Easier troubleshooting and debugging

## Testing Strategy

1. **Local Testing**: Run with `RUST_LOG=debug` to see detailed output
2. **Integration Testing**: Verify logs appear correctly in Docker containers
3. **Production Testing**: Use `RUST_LOG=info` for balanced visibility
4. **Performance Testing**: Ensure logging doesn't impact request processing

## Implementation Order

1. ✅ **Phase 1**: Catalog Service (Test Implementation)
2. **Phase 2**: Apply to remaining services (inventory, orders, price)
3. **Phase 3**: Add runtime logging improvements
4. **Phase 4+**: Advanced features based on Phase 1-3 results

## Success Metrics

- **Startup Issues**: Reduced time to identify connection problems
- **Operational Visibility**: Clear understanding of service state
- **Error Context**: Better error messages with relevant context
- **Performance**: Request processing timing visibility
- **Security**: No sensitive data exposure in logs
