# Rust Commerce

A microservices-based e-commerce platform built with Rust, featuring distributed architecture with NATS messaging, Protocol Buffers for inter-service communication, and MongoDB for persistence.

## Architecture Overview

This project implements a modular e-commerce system using modern microservices patterns:

- **Message-driven Architecture**: Services communicate asynchronously via NATS messaging
- **Protocol Buffers**: Type-safe, efficient serialization for inter-service communication
- **MongoDB**: Document-based persistence for flexible data modeling
- **Rust Workspace**: Organized as a Cargo workspace for shared dependencies and coordinated builds

## Services

### 1. Orders Service (`orders/`)

Handles order lifecycle management with the following capabilities:

- **Create Order**: Process new order creation with customer details and line items
- **Get Order**: Retrieve order information by ID
- **Delete Order**: Remove orders from the system
- **Order Items**: Manage individual line items within orders
- **Address Management**: Handle billing and shipping addresses
- **Order Totals**: Calculate and track order pricing and totals

**Key Components:**
- Protocol Buffers definitions in `proto/` for order domain models
- Service binary (`order-service`) listening on NATS queue `orders.*`
- Client binary (`order-client`) for testing and interaction
- MongoDB persistence layer with DAO pattern
- Handlers for business logic and request routing

### 2. Price Service (`price/`)

Manages pricing and offers with time-based validity:

- **Create Offer**: Define pricing offers with start/end dates and quantity ranges
- **Get Offer**: Retrieve pricing information by offer ID
- **Delete Offer**: Remove pricing offers
- **Price Management**: Handle multiple price points per offer
- **Quantity-based Pricing**: Support minimum/maximum quantity constraints

**Key Components:**
- Protocol Buffers definitions for offer and pricing models
- Service binary (`price-service`) for offer management
- Client binary (`price-client`) for testing
- MongoDB persistence with offer-specific collections
- Time-aware pricing logic with validity periods

### 3. Catalog Service (`catalog/`) ✅

Manages product catalog with comprehensive product information using a modern layered architecture:

- **Product Management**: Create, read, update, and delete products
- **Product Search**: Advanced search by name, description, category, and brand  
- **Variant Support**: Handle product variants with attributes (size, color, etc.)
- **Category Management**: Hierarchical category structures with tree caching
- **Product Slugs**: SEO-friendly URLs with automatic slug generation
- **Import/Export**: Bulk operations for product and category data
- **Type-Safe Domain Models**: Parse-don't-validate approach with enforced invariants

**Architecture Layers:**
- **Domain Layer**: Type-safe models with validation at construction (parse-don't-validate)
- **Handlers Layer**: NATS message processing and request routing
- **Services Layer**: Business logic and orchestration
- **Persistence Layer**: MongoDB data access with DAO pattern
- **Integration Tests**: Isolated test framework with per-test databases

### 4. Future Services

The `catalog/` directory contains the catalog service for:

- Product information management
- Variant handling (size, color, etc.)
- Category hierarchies
- Product attributes and metadata
- SEO and display configuration
- Search and filtering capabilities

## Technology Stack

### Core Technologies

- **Rust 2021 Edition**: Modern, safe systems programming
- **Tokio**: Async runtime for high-performance concurrent operations
- **NATS**: Lightweight, high-performance messaging system
- **MongoDB**: Document database for flexible schema design
- **Protocol Buffers (prost)**: Efficient, type-safe serialization

### Key Dependencies

- **async-nats**: NATS client with service pattern support
- **mongodb**: Official MongoDB driver for Rust
- **prost**: Protocol Buffers implementation
- **serde**: Serialization framework
- **uuid**: Unique identifier generation
- **iso_currency**: Currency handling
- **rust_decimal**: Precise decimal arithmetic
- **tokio**: Async runtime and utilities
- **clap**: Command-line interface framework

## Getting Started

### Prerequisites

1. **Rust**: Install from [rustup.rs](https://rustup.rs/)
2. **MongoDB**: Running instance (local or remote)
3. **NATS Server**: Download from [nats.io](https://nats.io/download/)

### Environment Setup

Create a `.env` file in the project root:

```env
MONGODB_URL=mongodb://localhost:27017
```

### Starting Infrastructure

1. **Start NATS Server**:
   ```bash
   nats-server
   ```
   Default port: 4222

2. **Start MongoDB**:
   ```bash
   mongod
   ```
   Default port: 27017

### Building the Project

Build all services in the workspace:

```bash
cargo build
```

Build specific service:

```bash
cargo build -p rust-orders
cargo build -p rust-price
cargo build -p rust-catalog
```

### Running Services

#### Orders Service

```bash
cargo run --bin order-service
```

The service will:
- Connect to NATS at `0.0.0.0:4222`
- Subscribe to queue `orders.*`
- Connect to MongoDB database `db_orders`

#### Price Service

```bash
cargo run --bin price-service
```

The service will:
- Connect to NATS for messaging
- Use MongoDB database `db_prices`

#### Catalog Service

```bash
cargo run --bin catalog-service
```

The service will:
- Connect to NATS for messaging
- Subscribe to queue `catalog.*`
- Use MongoDB database `db_catalog`

### Testing with Clients

#### Orders Client

```bash
cargo run --bin order-client
```

#### Price Client

```bash
cargo run --bin price-client
```

#### Catalog Client

```bash
cargo run --bin catalog-client -- --help
```

Example catalog operations:
```bash
# Create a product
cargo run --bin catalog-client -- product-create --name "Sample Product" --brand "Sample Brand"

# Search products
cargo run --bin catalog-client -- product-search --query "sample"
```

## Development

### Service Architecture Pattern

The catalog service implements a layered architecture that serves as the template for all services:

#### Architecture Layers

1. **Domain Layer** (`domain/`)
   - Type-safe domain models following "parse don't validate" principle
   - Value objects with validation at construction
   - Invalid states are unrepresentable in the type system
   - Example: `ProductName`, `ProductRef` with enforced constraints

2. **Handlers Layer** (`handlers/`)
   - NATS message processing and deserialization
   - Request routing to appropriate services
   - Protocol Buffer message conversion
   - Error response formatting

3. **Services Layer** (`services/`)
   - Business logic and orchestration
   - Transaction coordination
   - Cross-entity operations
   - Dependency injection via `Arc<AppState>`

4. **Persistence Layer** (`persistence/`)
   - MongoDB data access objects (DAOs)
   - Database operations and queries
   - Collection management
   - Index creation and optimization

#### Key Design Principles

- **Type-Based Design**: Domain models enforce invariants at compile time
- **Separation of Concerns**: Clear boundaries between layers
- **Dependency Injection**: Services and DAOs injected via AppState
- **Testability**: Each layer can be tested independently
- **Parse Don't Validate**: Validation happens once at boundaries

### Project Structure

```
rust_commerce/
├── Cargo.toml              # Workspace configuration
├── orders/                 # Orders microservice
│   ├── Cargo.toml         # Orders service dependencies
│   ├── build.rs           # Protocol buffer compilation
│   ├── proto/             # Protocol buffer definitions
│   └── src/
│       ├── order-service/ # Service implementation
│       └── order-client/  # Test client
├── price/                  # Price microservice
│   ├── Cargo.toml         # Price service dependencies
│   ├── build.rs           # Protocol buffer compilation
│   ├── proto/             # Protocol buffer definitions
│   └── src/
│       ├── price-service/ # Service implementation
│       └── price-client/  # Test client
├── catalog/               # Catalog microservice
│   ├── Cargo.toml         # Catalog service dependencies
│   ├── build.rs           # Protocol buffer compilation
│   ├── proto/             # Protocol buffer definitions
│   ├── src/
│   │   ├── catalog-service/ # Service implementation
│   │   └── catalog-client/  # Test client
│   ├── sample_records_backup/ # Original sample data
│   └── README.md          # Catalog service documentation
└── target/                # Build artifacts
```

### Adding New Services

1. Create new directory in workspace root
2. Add service to `Cargo.toml` workspace members
3. Define Protocol Buffer schemas in `proto/`
4. Implement service and client binaries
5. Add appropriate NATS subjects and handlers

### Protocol Buffer Development

Services use Protocol Buffers for message definition:

1. Define `.proto` files in service `proto/` directory
2. Update `build.rs` to compile new proto files
3. Generated Rust code available in build output
4. Use in service code via included modules

### Testing

#### Unit Tests

Run all tests:

```bash
cargo test
```

Run service-specific tests:

```bash
cargo test -p rust-orders
cargo test -p rust-price
cargo test -p rust-catalog
```

#### Integration Tests

The catalog service includes comprehensive integration tests with test isolation:

```bash
# Run catalog integration tests sequentially (recommended)
./scripts/test-catalog.sh

# Or manually with single thread
cargo test -p rust-catalog --test '*' -- --test-threads=1
```

**Test Infrastructure:**
- Each test gets its own MongoDB database for complete isolation
- Tests spawn the actual service using `spawn_app()` helper
- Automatic cleanup of test databases after completion
- `TestApp` helper in `rust-common` provides shared test utilities

#### Code Quality

Before committing code, ensure it passes formatting and linting:

```bash
# Format all code
cargo fmt --all

# Run clippy linter
cargo clippy --all-targets --all-features

# Combined check (recommended before commits)
cargo fmt --all && cargo clippy --all-targets --all-features
```

### Logging

Services use `pretty_env_logger` for structured logging. Set log level:

```bash
RUST_LOG=debug cargo run --bin order-service
```

## NATS Communication Patterns

### Message Routing

- **Orders**: Queue subscription on `orders.*`
- **Prices**: Subject-based routing for offer operations
- **Queue Groups**: Load balancing across service instances
- **Request-Reply**: Synchronous-style communication over async messaging

### Message Flow

1. Client sends protobuf-encoded request to NATS subject
2. Service receives message from queue subscription
3. Service processes request using business logic and persistence
4. Service sends protobuf-encoded response back via NATS
5. Client receives and processes response

## Database Schema

### Orders Database (`db_orders`)

- **Collection**: `orders`
- **Documents**: Order aggregates with embedded items and addresses

### Price Database (`db_prices`)

- **Collection**: `prices`
- **Documents**: Offer documents with time-based validity and pricing tiers

### Catalog Database (`db_catalog`)

- **Collection**: `products`
- **Documents**: Product documents with embedded variants, pricing, and metadata

## Contributing

1. Fork the repository
2. Create feature branch (`git checkout -b feature/amazing-feature`)
3. Commit changes (`git commit -m 'Add amazing feature'`)
4. Push to branch (`git push origin feature/amazing-feature`)
5. Open Pull Request

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Future Roadmap

- [x] Complete catalog service implementation with layered architecture
- [x] Add comprehensive integration tests for catalog service
- [x] Docker containerization with multi-stage builds
- [ ] Migrate inventory, orders, and price services to layered architecture
- [ ] Add authentication and authorization
- [ ] Implement inventory management service
- [ ] Add shopping cart functionality
- [ ] Create web API gateway
- [ ] Add monitoring and observability
- [ ] Implement event sourcing patterns
- [ ] Kubernetes deployment manifests
- [ ] Add distributed tracing with OpenTelemetry
