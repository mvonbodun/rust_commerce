# TypeScript Client Integration Tests

This directory contains integration tests for the TypeScript client library that verify communication with the Rust microservices.

## Test Structure

```
tests/
├── integration/         # Integration test suites
│   ├── product.test.ts  # Product API tests
│   └── category.test.ts # Category API tests
├── utils/              # Test utilities and helpers
│   └── test-client.ts  # Shared test client setup
├── setup.ts           # Jest setup and custom matchers
└── README.md          # This file
```

## Running Tests

### Prerequisites

1. Install dependencies:
   ```bash
   npm install
   ```

2. Generate TypeScript types from proto files:
   ```bash
   npm run generate
   ```

### Running Tests Locally

#### Option 1: Using Docker Compose (Recommended)

This starts all required services in Docker containers:

```bash
# Start services and run tests
npm run test:docker

# Or manually:
npm run docker:up      # Start test environment
npm run test:integration  # Run tests
npm run docker:down    # Stop test environment
```

#### Option 2: With Local Services

If you have MongoDB and NATS running locally:

```bash
# Start catalog service manually
MONGODB_URL=mongodb://admin:password@localhost:27017 \
NATS_URL=nats://localhost:4222 \
cargo run --bin catalog-service

# Run tests (in another terminal)
NATS_TEST_URL=nats://localhost:4222 npm run test:integration
```

#### Option 3: Using the Test Script

```bash
./scripts/test-with-docker.sh
```

### Test Coverage

Generate coverage report:

```bash
npm run test:coverage
```

Coverage reports are generated in `coverage/` directory.

## Test Environment

### Docker Compose Services

The `docker-compose.test.yml` file sets up:
- **MongoDB** on port 27018 (test database)
- **NATS** on port 4223 (messaging)
- **Catalog Service** (Rust microservice)

### Environment Variables

- `NATS_TEST_URL`: NATS server URL (default: `nats://localhost:4223`)
- `MONGODB_URL`: MongoDB connection string (used by catalog service)

## Writing Tests

### Test Utilities

The `test-client.ts` utility provides:
- `getTestClient()`: Returns a singleton test client
- `generateTestId()`: Creates unique test identifiers
- `waitFor()`: Waits for async conditions
- `retry()`: Retries operations with backoff

### Custom Jest Matchers

- `toBeValidProduct()`: Validates product structure
- `toBeValidCategory()`: Validates category structure

### Example Test

```typescript
import { Code } from '../../src';
import { getTestClient, generateTestId } from '../utils/test-client';

describe('My Feature', () => {
  let client;
  
  beforeAll(async () => {
    client = await getTestClient();
  });
  
  test('should do something', async () => {
    const response = await client.catalog.createProduct({
      name: 'Test Product',
      productRef: generateTestId('PROD'),
      // ... other fields
    });
    
    expect(response.status?.code).toBe(Code.OK);
    expect(response.product).toBeValidProduct();
  });
});
```

## Test Data Cleanup

Tests should clean up their data in `afterAll()` hooks:

```typescript
const createdIds = [];

afterAll(async () => {
  for (const id of createdIds) {
    await client.catalog.deleteProduct({ id });
  }
});
```

## CI/CD Integration

The GitHub Actions workflow (`.github/workflows/typescript-client.yml`) runs:
1. Build and type generation
2. Integration tests with services
3. Coverage reporting
4. NPM publishing (on main branch)

## Troubleshooting

### "Connection refused" errors
- Ensure Docker services are running: `docker-compose -f docker-compose.test.yml ps`
- Check service logs: `npm run docker:logs`

### "Timeout" errors
- Services may need more time to start
- Increase timeout in `jest.config.js`

### "Service not ready" errors
- Check catalog service logs: `docker-compose -f docker-compose.test.yml logs catalog-service-test`
- Ensure MongoDB and NATS are healthy

## Best Practices

1. **Use unique test data**: Always use `generateTestId()` for unique identifiers
2. **Clean up after tests**: Delete created resources in `afterAll()`
3. **Test error cases**: Include tests for validation errors and edge cases
4. **Keep tests isolated**: Each test should be independent
5. **Use descriptive names**: Test names should clearly describe what they test

## Coverage Goals

Aim for:
- ✅ All CRUD operations tested
- ✅ Error handling tested
- ✅ Validation rules tested
- ✅ Search and filtering tested
- ✅ Edge cases covered