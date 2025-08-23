# Proto-Driven API Architecture

## Phase 1 - Proto Infrastructure (✅ Completed)

### What We've Implemented

1. **Created `nats/options.proto`** with custom protobuf options for:
   - Service-level NATS configuration (queue, subject_prefix)
   - Method-level configuration (subject override, timeout)
   - Event definitions with publishers/consumers
   - Service metadata for documentation

2. **Updated Proto Files**:
   - **`category.proto`**: Added `CategoryService` with all category RPCs and NATS options
   - **`product.proto`**: 
     - Renamed `CatalogService` to `ProductService` with NATS options
     - Kept deprecated `CatalogService` for backward compatibility
   - **`events.proto`**: Created comprehensive event definitions for:
     - Product events (created, updated, deleted)
     - Category events (created, updated, deleted, tree rebuilt)
     - Bulk import events
     - External events consumed from other services

3. **Updated `build.rs`** to include all proto files for compilation

### Proto Configuration Examples

```proto
service ProductService {
    option (nats.options.queue) = "catalog-queue";
    option (nats.options.subject_prefix) = "catalog";
    
    rpc CreateProduct(ProductCreateRequest) returns (ProductCreateResponse) {
        option (nats.options.subject) = "create_product";
    }
}
```

## Phase 2 - Rust Code Generation (Next Steps)

### Goal
Generate NATS configuration from proto definitions instead of hardcoding in startup.rs

### Implementation Plan

1. **Create Custom Build Script** (`build_nats.rs`):
   ```rust
   // Parse proto files to extract service metadata
   // Generate nats_config.rs with:
   pub mod catalog {
       pub const QUEUE: &str = "catalog-queue";
       pub const SUBJECT_PREFIX: &str = "catalog";
       
       pub mod subjects {
           pub const CREATE_PRODUCT: &str = "catalog.create_product";
           pub const GET_PRODUCT: &str = "catalog.get_product";
           // ... other subjects
       }
       
       pub mod events {
           pub const PRODUCT_CREATED: &str = "catalog.events.product.created";
           // ... other events
       }
   }
   ```

2. **Update `startup.rs`**:
   - Replace hardcoded `"catalog.*"` with `catalog::QUEUE` and `catalog::SUBJECT_PREFIX`
   - Use generated subject constants in router setup
   - Use event constants for publishing

3. **Create Event Publisher Helper**:
   ```rust
   impl AppState {
       pub async fn publish_product_created(&self, event: ProductCreatedEvent) {
           self.nats_client.publish(
               catalog::events::PRODUCT_CREATED,
               event.encode_to_vec().into()
           ).await
       }
   }
   ```

### Technical Approach

Since Prost doesn't natively support extracting custom options, we have several options:

**Option 1: Use protoc with custom plugin**
- Write a protoc plugin that reads the custom options
- Generate Rust code with NATS configuration
- Integrate into build.rs

**Option 2: Parse proto files directly**
- Use a proto parser library (e.g., `protobuf-parse`)
- Extract service/method options manually
- Generate Rust code

**Option 3: Use prost-reflect (Recommended)**
- Use `prost-reflect` to load descriptors at build time
- Extract custom options from descriptors
- Generate configuration code

## Phase 3 - TypeScript Client Library

### Structure
```
packages/rust-commerce-client/
├── src/
│   ├── generated/          # Proto-generated TypeScript
│   ├── nats/
│   │   ├── client.ts       # NATS connection wrapper
│   │   └── subjects.ts     # Generated subject constants
│   ├── services/
│   │   ├── product.ts      # Product service client
│   │   └── category.ts     # Category service client
│   └── events/
│       ├── subscriber.ts   # Event subscription handler
│       └── types.ts        # Event type definitions
└── package.json
```

### Client Usage Example
```typescript
import { CatalogClient } from '@rust-commerce/client';

const client = new CatalogClient('nats://localhost:4222');

// RPC calls
const product = await client.products.create({
    name: "Test Product",
    productRef: "TEST-001"
});

// Event subscriptions
client.events.on('ProductCreated', (event) => {
    console.log('New product:', event.productId);
});
```

### Generation Tools
- **protobuf-ts**: Generate TypeScript from proto files
- **ts-proto**: Alternative with better async/await support
- Custom wrapper for NATS.js client

## Phase 4 - Documentation & Testing

### Auto-generated Documentation
1. Extract service metadata and method descriptions from proto
2. Generate OpenAPI/AsyncAPI specifications
3. Create interactive API documentation

### Testing Infrastructure
1. Integration tests using generated clients
2. Contract testing between services
3. Event stream validation

## Benefits Achieved

1. **Single Source of Truth**: Proto files define everything
2. **No Hardcoded Subjects**: All routing from proto definitions
3. **Event Documentation**: Clear event contracts
4. **Type Safety**: End-to-end from Rust to TypeScript
5. **Service Discovery**: Services self-describe their capabilities

## Current Status

✅ **Phase 1 Complete**: Proto infrastructure is in place
⏳ **Phase 2 Ready**: Can now implement code generation
⏳ **Phase 3 Planned**: TypeScript client generation ready to start
⏳ **Phase 4 Planned**: Documentation and testing framework

## Next Immediate Steps

1. Choose proto parsing approach (recommend prost-reflect)
2. Implement build script for NATS config generation
3. Update startup.rs to use generated configuration
4. Test that all existing functionality still works
5. Begin TypeScript client development

## Migration Path

The current implementation remains fully functional. The migration to generated configuration can be done incrementally:

1. Generate configuration alongside hardcoded values
2. Gradually replace hardcoded values with generated ones
3. Remove hardcoded values once everything uses generated config
4. Maintain backward compatibility throughout