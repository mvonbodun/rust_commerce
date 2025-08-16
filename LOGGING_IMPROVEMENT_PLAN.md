# Logging & Tracing Improvement Plan

## Overview
This plan outlines comprehensive improvements to logging and tracing across the Rust Commerce microservices, with a focus on enhanced visibility during startup, runtime operations, and error handling.

## Current State Analysis
- ✅ Basic `pretty_env_logger::init()` setup
- ✅ Minimal startup logging
- ❌ No MongoDB connection logging
- ❌ No NATS connection logging
- ❌ No database/collection setup logging
- ❌ No index creation progress logging
- ❌ No service initialization logging
- ❌ No error context in connection failures

## Implementation Phases

### Phase 1: Enhanced Startup Logging ⭐ **PRIORITY 1**

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

### Phase 2: Service Initialization Logging ⭐ **PRIORITY 2**

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

### Phase 3: Runtime Logging ⭐ **PRIORITY 2**

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

### Phase 4: Error Context & Health Monitoring (Future)

#### 4.1 Connection Health Checks
- Periodic MongoDB health checks
- NATS connection monitoring
- Service dependency validation

#### 4.2 Graceful Shutdown Logging
- Signal handling for graceful shutdown
- Resource cleanup logging
- Service state preservation

### Phase 5: Utility Functions (Future)

#### 5.1 URL Masking for Security
```rust
fn mask_sensitive_url(url: &str) -> String {
    // Implementation for masking passwords in connection strings
}
```

### Phase 6: Log Level Recommendations (Future)

#### 6.1 Environment-Specific Logging
```bash
# Development
RUST_LOG=debug,catalog_service=trace

# Production  
RUST_LOG=info,mongodb=warn,async_nats=warn

# Troubleshooting
RUST_LOG=trace,hyper=debug,mongodb=debug
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
