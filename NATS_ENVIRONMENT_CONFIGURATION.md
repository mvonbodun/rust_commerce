# NATS Environment Configuration Update

## Overview

Updated all rust_commerce modules (catalog, inventory, orders, price) to use environment variables for NATS server configuration instead of hardcoded URLs. This enables easier deployment across different environments.

## Changes Made

### 1. Environment Files Updated

Added `NATS_URL` to all module `.env` files:

#### `/catalog/.env`
```properties
MONGODB_URL=mongodb://localhost:27017/catalog
NATS_URL=nats://localhost:4222
```

#### `/inventory/.env`
```properties
MONGODB_URL=mongodb://localhost:27017/inventory
NATS_URL=nats://localhost:4222
```

#### `/orders/.env`
```properties
MONGODB_URL=mongodb://localhost:27017/orders
NATS_URL=nats://localhost:4222
```

#### `/price/.env`
```properties
MONGODB_URL=mongodb://localhost:27017/price
NATS_URL=nats://localhost:4222
```

### 2. Service Components Updated

All service main.rs files updated to read NATS URL from environment:

- `catalog/src/catalog-service/main.rs`
- `inventory/src/inventory-service/main.rs` 
- `orders/src/order-service/main.rs`
- `price/src/price-service/main.rs`

**Pattern Applied:**
```rust
// Get NATS URL
let nats_url = env::var("NATS_URL").unwrap_or_else(|_| "nats://localhost:4222".to_string());

// Connect to the nats server
let nats_client = async_nats::connect(&nats_url).await?;
```

### 3. Client Components Updated

All client main.rs files updated to read NATS URL from environment:

- `catalog/src/catalog-client/main.rs`
- `inventory/src/inventory-client/main.rs`
- `orders/src/order-client/main.rs`
- `price/src/price-client/main.rs`

**Pattern Applied:**
```rust
// Load environment variables
dotenv().ok(); // or dotenvy::dotenv().ok() for some modules

// Get NATS URL from environment
let nats_url = std::env::var("NATS_URL").unwrap_or_else(|_| "nats://localhost:4222".to_string());

// Connect to the nats server
let client = async_nats::connect(&nats_url).await?;
```

## Benefits

1. **Environment Flexibility**: Can now deploy to different environments (dev, staging, production) by simply changing environment variables
2. **Container-Ready**: Docker deployments can override NATS URLs via environment variables
3. **Local Development**: Maintains backward compatibility with default localhost:4222
4. **Configuration Management**: Centralized configuration through environment variables
5. **Security**: No hardcoded URLs in source code

## Default Behavior

- If `NATS_URL` environment variable is not set, defaults to `nats://localhost:4222`
- This maintains backward compatibility with existing setups
- All modules use consistent environment variable naming

## Usage Examples

### Local Development (Default)
```bash
cargo run --bin catalog-service
# Uses NATS_URL from .env file: nats://localhost:4222
```

### Custom Environment
```bash
NATS_URL=nats://production-nats:4222 cargo run --bin catalog-service
```

### Docker Deployment
```yaml
environment:
  - MONGODB_URL=mongodb://mongo-cluster:27017/catalog
  - NATS_URL=nats://nats-cluster:4222
```

## Testing Verification

✅ All modules compile successfully  
✅ Services and clients read NATS_URL from environment  
✅ Default fallback to localhost:4222 works  
✅ Custom NATS URLs are properly applied  
✅ DNS errors occur when invalid URLs provided (confirming environment reading)

## Files Modified

### Environment Files
- `catalog/.env`
- `inventory/.env` 
- `orders/.env`
- `price/.env`

### Service Files
- `catalog/src/catalog-service/main.rs`
- `inventory/src/inventory-service/main.rs`
- `orders/src/order-service/main.rs`
- `price/src/price-service/main.rs`

### Client Files
- `catalog/src/catalog-client/main.rs`
- `inventory/src/inventory-client/main.rs`
- `orders/src/order-client/main.rs`
- `price/src/price-client/main.rs`

## Implementation Notes

- Used consistent error handling with `unwrap_or_else` for graceful fallback
- Added proper environment variable loading (`dotenv()` or `dotenvy::dotenv()`)
- Maintained existing code patterns and structure
- All changes are backward compatible
