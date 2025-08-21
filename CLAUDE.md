# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Development Commands

### Build Commands
```bash
# Build entire workspace
cargo build

# Build specific service
cargo build -p rust-catalog
cargo build -p rust-inventory
cargo build -p rust-orders
cargo build -p rust-price

# Build release version
cargo build --release

# Build specific binary
cargo build --bin catalog-service
cargo build --bin catalog-client
```

### Test Commands
```bash
# Run all tests in workspace
cargo test

# Run tests for specific service
cargo test -p rust-catalog
cargo test -p rust-inventory
cargo test -p rust-orders
cargo test -p rust-price

# Run specific test
cargo test test_name

# Run tests with output
cargo test -- --nocapture

# Run integration tests only
cargo test --test '*'
```

### Lint and Format Commands
```bash
# Format all code in workspace
cargo fmt --all

# Format specific package
cargo fmt -p rust-catalog

# Check formatting without applying
cargo fmt --all -- --check

# Run clippy linter on entire workspace
cargo clippy --all-targets --all-features

# Run clippy with automatic fixes
cargo clippy --fix --allow-dirty --allow-staged

# Run clippy and fail on warnings
cargo clippy --all-targets --all-features -- -D warnings

# Development workflow - run before committing
cargo fmt --all && cargo clippy --all-targets --all-features
```

### Running Services
```bash
# Start individual services
cargo run --bin catalog-service
cargo run --bin inventory-service
cargo run --bin order-service
cargo run --bin price-service

# Start test clients
cargo run --bin catalog-client
cargo run --bin inventory-client
cargo run --bin order-client
cargo run --bin price-client

# With environment variables
RUST_LOG=debug cargo run --bin catalog-service
```

### Docker Commands
```bash
# Start all services with docker-compose
docker-compose up

# Start specific service
docker-compose up catalog-service

# Build Docker images
docker-compose build

# View logs
docker-compose logs -f catalog-service
```

### Fly.io Deployment
```bash
# Deploy single service
./scripts/fly-setup.sh catalog production

# Deploy all services
./scripts/fly-setup-all.sh production

# Monitor services
./scripts/fly-monitor.sh --watch

# Deploy with parallel execution
./scripts/fly-start-stop-all.sh
```

## Architecture Overview

This is a microservices-based e-commerce platform with the following architecture:

### Core Components

1. **Message Bus**: NATS server provides asynchronous messaging between services
   - Services subscribe to subject patterns (e.g., `catalog.*`, `orders.*`)
   - Request-reply pattern for synchronous-style communication
   - Queue groups for load balancing across instances

2. **Data Layer**: MongoDB for document-based persistence
   - Each service has its own database (`db_catalog`, `db_inventory`, `db_orders`, `db_prices`)
   - DAO pattern for data access abstraction
   - BSON serialization for flexible schemas

3. **Service Communication**: Protocol Buffers for type-safe messaging
   - Proto definitions in each service's `proto/` directory
   - Compiled at build time via `build.rs`
   - Shared status and error code definitions

### Service Architecture

Each microservice follows a consistent structure:

```
service/
├── src/
│   ├── lib.rs                    # Shared library code
│   ├── service-name-service/     # Service implementation
│   │   ├── main.rs               # Service entry point, NATS setup
│   │   ├── handlers/             # Request handlers
│   │   │   ├── mod.rs           # Handler registration
│   │   │   └── handlers_inner.rs # Business logic
│   │   ├── model.rs             # Domain models
│   │   └── persistence/         # Data access layer
│   │       ├── mod.rs
│   │       └── *_dao.rs         # MongoDB operations
│   └── service-name-client/      # Test client
│       └── main.rs               # CLI for testing
├── proto/                        # Protocol Buffer definitions
├── build.rs                      # Proto compilation
└── tests/                        # Integration tests
```

### Service Responsibilities

- **Catalog Service**: Product management, categories, search, variants
- **Inventory Service**: Stock levels, location-based inventory, SKU management
- **Orders Service**: Order lifecycle, line items, addresses, totals
- **Price Service**: Offers, time-based pricing, quantity tiers

### Key Patterns

1. **Handler Pattern**: Each service operation has a dedicated handler function
2. **DAO Pattern**: Data access objects encapsulate MongoDB operations
3. **Error Handling**: Consistent error types with Protocol Buffer status codes
4. **Async/Await**: Tokio runtime for concurrent request handling
5. **Environment Configuration**: Services read MONGODB_URL and NATS_URL from environment

### Inter-Service Communication Flow

1. Client sends protobuf request to NATS subject (e.g., `catalog.product.create`)
2. Service handler receives message from queue subscription
3. Handler validates request and calls DAO for persistence
4. DAO performs MongoDB operations
5. Handler constructs protobuf response with status
6. Response sent back through NATS to waiting client

### Deployment Architecture

- **Local**: Docker Compose orchestration with MongoDB and NATS containers
- **Fly.io**: Distributed deployment with services as Fly machines
  - Internal networking via `.internal` domain
  - External IPs for fallback connectivity
  - Secrets management for connection strings

## Important Considerations

- Services are stateless and can be scaled horizontally
- Each service manages its own database schema
- NATS provides both messaging and service discovery
- Protocol Buffers ensure backward compatibility
- Integration tests use real MongoDB and NATS instances
- Fly.io deployment uses multi-strategy connection logic for reliability