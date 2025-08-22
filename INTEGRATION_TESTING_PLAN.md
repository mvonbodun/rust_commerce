# Integration Testing Improvement Plan

## Overview
This document outlines the comprehensive plan for improving integration testing across all modules in the rust_commerce project. The approach follows principles from "Zero to Production" and industry best practices for Rust testing.

## Goals
- 100% coverage of all client commands (public API)
- Both positive and negative test cases for each command
- Shared test infrastructure to reduce duplication
- Fast, reliable, and maintainable tests
- Clear documentation and patterns for adding new tests

## Test Structure

### Directory Organization
Each module will follow this structure:
```
module/tests/
├── api/                    # Integration tests
│   ├── helpers/           # Shared test utilities
│   │   ├── mod.rs
│   │   ├── test_app.rs   # Test application setup
│   │   └── fixtures.rs   # Test data factories
│   ├── commands/          # One file per command group
│   │   ├── product_tests.rs
│   │   ├── category_tests.rs
│   │   └── ...
│   └── main.rs           # Test entry point
└── unit/                  # Unit tests (if any)
```

## Shared Test Infrastructure

### Core Components

#### 1. TestApp Structure
```rust
pub struct TestApp {
    pub nats_client: async_nats::Client,
    pub mongodb_client: mongodb::Client,
    pub test_db_name: String,
}
```
- Manages service lifecycle
- Handles setup and teardown
- Provides isolated test environment

#### 2. Test Data Builders
- Valid data generators using `fake` crate
- Invalid data generators for negative testing
- Edge case data (boundary values, special characters)

#### 3. Assertion Helpers
- Response validation utilities
- Error checking helpers
- Performance measurement tools

## Test Coverage Matrix

### Catalog Module

#### Product Commands
| Command | Positive Tests | Negative Tests |
|---------|---------------|----------------|
| ProductCreate | - Valid product with all fields<br>- Minimal required fields<br>- With variants | - Missing required fields<br>- Invalid characters in name<br>- Duplicate product_ref |
| ProductGet | - Existing product by ID<br>- Recently created product | - Non-existent ID<br>- Invalid ID format<br>- Empty ID |
| ProductGetBySlug | - Valid slug<br>- Special characters in slug | - Non-existent slug<br>- Empty slug<br>- SQL injection attempts |
| ProductDelete | - Delete existing product<br>- Idempotent delete | - Non-existent product<br>- Invalid ID |
| ProductSearch | - Search by name<br>- Search by category<br>- Search by brand | - SQL injection<br>- XSS attempts<br>- Empty search |
| Import | - Valid JSON file<br>- Large batch import | - Invalid JSON<br>- Missing required fields<br>- Corrupted file |
| Export | - Export all products<br>- Export with pagination | - Invalid batch size<br>- Permission errors |

#### Category Commands
| Command | Positive Tests | Negative Tests |
|---------|---------------|----------------|
| CategoryCreate | - Root category<br>- Child category<br>- With SEO data | - Missing name<br>- Invalid parent_id<br>- Duplicate slug |
| CategoryGet | - Existing category<br>- With children | - Non-existent ID<br>- Invalid format |
| CategoryGetBySlug | - Valid slug<br>- Nested category | - Non-existent<br>- Invalid characters |
| CategoryUpdate | - Update name<br>- Update parent<br>- Partial update | - Invalid data<br>- Non-existent ID |
| CategoryDelete | - Leaf category<br>- With cascade | - Has children<br>- Non-existent |
| CategoryTree | - Full tree<br>- Subtree | - Empty tree<br>- Corrupted cache |
| Import/Export | - Valid hierarchy<br>- Large dataset | - Circular references<br>- Invalid format |

### Inventory Module

| Command | Positive Tests | Negative Tests |
|---------|---------------|----------------|
| Create | - Valid SKU and location<br>- With min stock level | - Duplicate SKU/location<br>- Negative quantity<br>- Invalid SKU format |
| Get | - Existing item<br>- Multiple locations | - Non-existent SKU<br>- Invalid format |
| UpdateStock | - Increase stock<br>- Decrease stock<br>- Reserve stock | - Insufficient stock<br>- Negative values<br>- Concurrent updates |
| Delete | - Existing item<br>- Already deleted | - Non-existent<br>- In-use item |
| GetAllLocationsBySku | - Single SKU<br>- Multiple SKUs<br>- With aggregation | - Empty list<br>- Too many SKUs<br>- Invalid SKUs |
| Import | - Valid CSV<br>- Batch update | - Invalid format<br>- Duplicate entries |

### Orders Module

| Command | Positive Tests | Negative Tests |
|---------|---------------|----------------|
| OrderCreate | - Valid order with items<br>- With shipping address | - Missing address<br>- Invalid items<br>- Zero quantities |
| OrderGet | - Existing order<br>- With line items | - Non-existent<br>- Invalid ID |
| OrderDelete | - Existing order<br>- Soft delete | - Non-existent<br>- Already shipped |
| OrderUpdate | - Update status<br>- Add items<br>- Update address | - Invalid status transition<br>- Invalid items |
| AddItem | - Valid SKU<br>- Update quantity | - Invalid SKU<br>- Zero quantity |
| RemoveItem | - Existing item<br>- Last item | - Non-existent item |

### Price Module

| Command | Positive Tests | Negative Tests |
|---------|---------------|----------------|
| OfferCreate | - Valid date range<br>- With tiers<br>- Multiple currencies | - Invalid dates<br>- Overlapping offers<br>- Past dates |
| OfferGet | - Active offer<br>- Future offer | - Non-existent<br>- Expired offer |
| OfferDelete | - Existing offer<br>- Future offer | - Non-existent<br>- Active offer |
| GetBestOfferPrice | - Single SKU<br>- With quantity | - No offers<br>- Invalid SKU |
| GetBestOfferPrices | - Multiple SKUs<br>- Mixed currencies | - Empty list<br>- Invalid data |

## Negative Test Patterns

### Data Validation Tests
- Empty strings: `""`
- Null values: `null`, `\0`
- Special characters: `!@#$%^&*()_+{}[]|\\:";'<>?,./`
- Unicode edge cases: emoji, RTL text, combining characters
- Very long strings: >1MB
- Whitespace only: `"   "`

### Security Tests
- SQL injection: `'; DROP TABLE--`, `1 OR 1=1`
- XSS attempts: `<script>alert('xss')</script>`
- Path traversal: `../../../etc/passwd`
- Command injection: `; ls -la`

### Boundary Tests
- Zero values: `0`, `-0`
- Negative values: `-1`, `MIN_INT`
- Maximum values: `MAX_INT`, `MAX_UINT`
- Floating point edge cases: `NaN`, `Infinity`

## Implementation Phases

### Phase 1: Test Infrastructure (Current)
- [x] Create shared test helpers module
- [ ] Implement TestApp pattern
- [ ] Create test data builders
- [ ] Setup assertion helpers

### Phase 2: Catalog Module
- [ ] Product command tests (all CRUD operations)
- [ ] Category command tests (hierarchy operations)
- [ ] Import/Export tests
- [ ] Property-based tests

### Phase 3: Inventory Module
- [ ] Basic CRUD tests
- [ ] Stock management tests
- [ ] Concurrency tests
- [ ] Batch operation tests

### Phase 4: Orders & Price Modules
- [ ] Orders: Full lifecycle tests
- [ ] Orders: Status transition tests
- [ ] Price: Offer management tests
- [ ] Price: Best price calculation tests

### Phase 5: Integration & Performance
- [ ] Cross-module workflows
- [ ] Performance benchmarks
- [ ] Load testing
- [ ] Documentation

## Test Execution

### Running Tests
```bash
# Run all tests
cargo test --workspace

# Run specific module tests
cargo test -p rust-catalog

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_name

# Run with nextest (faster)
cargo nextest run
```

### CI/CD Integration
- Tests run on every PR
- Coverage reports generated
- Performance regression detection
- Parallel execution with cargo-nextest

## Success Metrics
- ✅ 100% client command coverage
- ✅ Each command has 3+ positive, 3+ negative tests
- ✅ All tests complete in <30 seconds
- ✅ Zero flaky tests
- ✅ Test coverage >80% for public APIs
- ✅ Clear documentation for adding new tests

## Best Practices

### Test Isolation
- Each test gets a fresh database state
- Use unique identifiers (UUIDs) for test data
- Clean up resources in test teardown
- No shared mutable state between tests

### Test Naming
- Use descriptive names: `test_product_create_with_valid_data`
- Group related tests in modules
- Prefix negative tests: `test_product_create_fails_with_missing_name`

### Test Documentation
- Add doc comments explaining test purpose
- Include examples of expected behavior
- Document any special setup requirements

### Performance
- Use `#[ignore]` for slow tests
- Run expensive tests separately
- Mock external services when appropriate
- Use test fixtures to avoid repeated setup

## Adding New Tests

### Template for New Test
```rust
#[tokio::test]
async fn test_command_scenario() {
    // Arrange
    let app = TestApp::spawn().await;
    let test_data = fixtures::valid_product();
    
    // Act
    let response = app.execute_command(test_data).await;
    
    // Assert
    assert_eq!(response.status, StatusCode::OK);
    assert!(response.body.contains("expected"));
    
    // Cleanup handled by Drop trait
}
```

## Maintenance

### Regular Tasks
- Review and update test data monthly
- Check for new attack vectors quarterly
- Update dependencies regularly
- Monitor test execution times
- Remove obsolete tests

### Documentation Updates
- Keep this plan updated with new patterns
- Document any test utilities added
- Update coverage reports
- Share learnings from test failures