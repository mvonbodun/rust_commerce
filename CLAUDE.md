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

This is a microservices-based e-commerce platform with a layered architecture pattern established in the catalog service and being adopted across all services.

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

The catalog service establishes the layered architecture pattern that all services should follow:

```
service/
├── src/
│   ├── lib.rs                      # Shared library code, AppState definition
│   ├── service-name-service/       # Service implementation
│   │   ├── main.rs                 # Service entry point
│   │   ├── startup.rs              # Application initialization, routing setup
│   │   ├── validation.rs           # Service-specific validation
│   │   ├── domain/                 # Domain layer (type-based design)
│   │   │   ├── mod.rs             # Domain exports
│   │   │   ├── model.rs           # Core domain entities
│   │   │   ├── product_name.rs    # Value object with validation
│   │   │   └── product_ref.rs     # Value object with constraints
│   │   ├── handlers/               # Message processing layer
│   │   │   ├── mod.rs             # Router and handler registration
│   │   │   ├── product_handlers.rs # Product message handlers
│   │   │   └── category_handlers.rs # Category message handlers
│   │   ├── services/               # Business logic layer
│   │   │   ├── mod.rs             # Service exports
│   │   │   ├── product_service.rs # Product business logic
│   │   │   └── category_service.rs # Category business logic
│   │   └── persistence/            # Data access layer
│   │       ├── mod.rs             # DAO trait definitions
│   │       ├── product_dao.rs     # Product MongoDB operations
│   │       └── category_dao.rs    # Category MongoDB operations
│   └── service-name-client/        # Test client
│       └── main.rs                 # CLI for testing
├── proto/                          # Protocol Buffer definitions
│   ├── product.proto              # Product messages
│   └── category.proto             # Category messages
├── build.rs                        # Proto compilation
└── tests/                          # Integration tests
    └── api/                        # API integration tests
        ├── main.rs                # Test module setup
        ├── helpers/               # Test infrastructure
        │   ├── mod.rs            # Helper exports
        │   └── spawn_app.rs      # Test app spawner
        ├── product_tests.rs       # Product API tests
        └── category_tests.rs      # Category API tests
```

### Service Responsibilities

- **Catalog Service**: Product management, categories, search, variants
- **Inventory Service**: Stock levels, location-based inventory, SKU management
- **Orders Service**: Order lifecycle, line items, addresses, totals
- **Price Service**: Offers, time-based pricing, quantity tiers

### Key Patterns

1. **Layered Architecture**: Clear separation between domain, handlers, services, and persistence
2. **Type-Based Design**: Domain models enforce invariants at compile time (parse don't validate)
3. **Handler Pattern**: Message processing layer handles NATS communication
4. **Service Layer**: Business logic separated from message handling
5. **DAO Pattern**: Data access objects encapsulate MongoDB operations
6. **Dependency Injection**: Services and DAOs injected via Arc<AppState>
7. **Error Handling**: Consistent error types with Protocol Buffer status codes
8. **Async/Await**: Tokio runtime for concurrent request handling
9. **Environment Configuration**: Services read MONGODB_URL and NATS_URL from environment

### Inter-Service Communication Flow

1. Client sends protobuf request to NATS subject (e.g., `catalog.create_product`)
2. Router in handlers layer receives message from queue subscription
3. Handler deserializes protobuf and calls appropriate service method
4. Service layer executes business logic, coordinating with DAOs
5. DAO performs MongoDB operations
6. Service returns domain model to handler
7. Handler converts to protobuf response with status
8. Response sent back through NATS to waiting client

### Deployment Architecture

- **Local**: Docker Compose orchestration with MongoDB and NATS containers
- **Fly.io**: Distributed deployment with services as Fly machines
  - Internal networking via `.internal` domain
  - External IPs for fallback connectivity
  - Secrets management for connection strings

## Integration Testing

The catalog service includes a comprehensive integration testing framework that should be adopted for all services:

### Test Infrastructure

```bash
# Run catalog tests sequentially (required for proper isolation)
./scripts/test-catalog.sh

# Or manually
cargo test -p rust-catalog --test '*' -- --test-threads=1
```

### Key Testing Principles

1. **Test Isolation**: Each test gets its own MongoDB database
2. **Real Service Testing**: Tests spawn the actual service using `spawn_app()`
3. **Automatic Cleanup**: Test databases are dropped after completion
4. **Shared Utilities**: `TestApp` in rust-common provides test helpers
5. **Sequential Execution**: Tests must run sequentially to avoid NATS conflicts

### Test Organization

```
tests/api/
├── helpers/
│   └── spawn_app.rs    # Spawns isolated test instance
├── product_tests.rs    # Product API integration tests
└── category_tests.rs   # Category API integration tests
```

## Code Quality Standards

### Before Committing

Always run formatting and linting before commits:

```bash
# Format all code
cargo fmt --all

# Check for linting issues
cargo clippy --all-targets --all-features

# Combined (recommended)
cargo fmt --all && cargo clippy --all-targets --all-features
```

### Code Review Checklist

1. ✅ All tests pass (`cargo test`)
2. ✅ Code is formatted (`cargo fmt --all`)
3. ✅ No clippy warnings (`cargo clippy --all-targets --all-features`)
4. ✅ Domain models validate inputs (parse don't validate)
5. ✅ Error handling is comprehensive
6. ✅ Integration tests cover new functionality

## Architecture Migration Plan

### Current Status

- ✅ **Catalog Service**: Fully refactored to layered architecture
- ⏳ **Inventory Service**: Pending migration
- ⏳ **Orders Service**: Pending migration  
- ⏳ **Price Service**: Pending migration

### Migration Guidelines

When refactoring services to the new architecture:

1. **Start with Domain Layer**
   - Create value objects for validated types
   - Implement parse-don't-validate pattern
   - Move validation to type constructors

2. **Separate Handlers from Business Logic**
   - Handlers only process messages
   - Business logic moves to service layer
   - Services get injected via AppState

3. **Create Service Layer**
   - One service struct per entity type
   - Services hold Arc references to DAOs
   - Business methods on service structs

4. **Update Startup Pattern**
   - Use startup.rs for initialization
   - Centralize routing in handlers/mod.rs
   - Move validation to validation.rs

5. **Add Integration Tests**
   - Use spawn_app pattern from catalog
   - Ensure test isolation with unique databases
   - Run tests sequentially

## Important Considerations

- Services are stateless and can be scaled horizontally
- Each service manages its own database schema
- NATS provides both messaging and service discovery
- Protocol Buffers ensure backward compatibility
- Integration tests use real MongoDB and NATS instances
- Fly.io deployment uses multi-strategy connection logic for reliability
- **Follow the catalog service architecture pattern for all new development**