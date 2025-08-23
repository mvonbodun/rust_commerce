# Rust Commerce TypeScript Client

A TypeScript client library for interacting with Rust Commerce microservices via NATS.

## Features

- 🚀 Auto-generated TypeScript types from Protocol Buffers
- 📡 NATS-based communication with microservices
- 🔒 Type-safe request/response handling
- ⚡ Async/await support
- 🎯 Proto-driven API configuration

## Installation

```bash
npm install @rust-commerce/client
```

## Usage

### Basic Example

```typescript
import { createClient, Code } from '@rust-commerce/client';

// Create and connect to NATS
const client = await createClient({
  servers: 'nats://localhost:4222',
});

// Create a product
const response = await client.catalog.createProduct({
  name: 'Example Product',
  productRef: 'PROD-001',
  slug: 'example-product',
  // ... other fields
});

if (response.status?.code === Code.OK) {
  console.log('Product created:', response.product?.id);
}

// Don't forget to disconnect when done
await client.natsClient.disconnect();
```

### Advanced Configuration

```typescript
const client = await createClient({
  servers: ['nats://server1:4222', 'nats://server2:4222'],
  maxReconnectAttempts: 10,
  reconnectTimeWait: 5000,
  token: 'your-auth-token',
  user: 'username',
  pass: 'password',
});
```

## Available Services

### Catalog Service

#### Product Operations
- `createProduct(request)` - Create a new product
- `getProduct(request)` - Get product by ID
- `getProductBySlug(request)` - Get product by slug
- `updateProduct(request)` - Update an existing product
- `deleteProduct(request)` - Delete a product
- `searchProducts(request)` - Search for products
- `exportProducts(request)` - Export products
- `getProductSlugs(request)` - Get all product slugs

#### Category Operations
- `createCategory(request)` - Create a new category
- `getCategory(request)` - Get category by ID
- `getCategoryBySlug(request)` - Get category by slug
- `updateCategory(request)` - Update an existing category
- `deleteCategory(request)` - Delete a category
- `getCategoryTree(request)` - Get the category tree structure

## Development

### Generate Types from Proto Files

```bash
npm run generate
```

This will:
1. Generate TypeScript types from `.proto` files
2. Generate NATS configuration from service definitions

### Build the Library

```bash
npm run build
```

### Run Tests

```bash
npm test
```

## Proto-Driven Architecture

This client is automatically generated from Protocol Buffer definitions. The NATS routing configuration is extracted from the proto files at build time, ensuring the client always matches the server API.

### Proto Service Definition Example

```protobuf
service ProductService {
    option (nats.options.queue) = "catalog-queue";
    option (nats.options.subject_prefix) = "catalog";
    
    rpc CreateProduct(ProductCreateRequest) returns (ProductCreateResponse) {
        option (nats.options.subject) = "create_product";
    }
}
```

This generates:
- TypeScript types for `ProductCreateRequest` and `ProductCreateResponse`
- NATS configuration with subject `catalog.create_product`
- Type-safe client method `createProduct()`

## License

MIT