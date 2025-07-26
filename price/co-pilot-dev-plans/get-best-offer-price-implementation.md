# Plan: GetBestOfferPrice API Implementation

## Overview
Implement a new API endpoint `GetBestOfferPrice` that finds the lowest-priced offer for a given SKU, quantity, and currency, optionally filtered by date.

## 1. Protocol Buffer Schema Changes

### File: `price/proto/offer.proto`
- Add new message `GetBestOfferPriceRequest`:
  ```protobuf
  message GetBestOfferPriceRequest {
    string sku = 1;
    int32 quantity = 2;
    optional string date = 3; // ISO 8601 format, defaults to current date
    string currency = 4;
  }
  ```
- Add new message `GetBestOfferPriceResponse`:
  ```protobuf
  message GetBestOfferPriceResponse {
    optional Offer offer = 1;
    bool found = 2;
  }
  ```
- Add RPC method to service:
  ```protobuf
  rpc GetBestOfferPrice(GetBestOfferPriceRequest) returns (GetBestOfferPriceResponse);
  ```

## 2. Handler Implementation

### File: `price/src/price-service/handlers/get_best_offer_price.rs` (new)
- **Input Validation**:
  - Validate `sku` is not empty
  - Validate `quantity` is positive integer
  - Validate `currency` is valid ISO currency code (USD/EUR)
  - Validate optional `date` is valid ISO 8601 format
  - Return `Status::InvalidArgument` with descriptive error for validation failures

- **Date Handling**:
  - If date not provided, use `chrono::Utc::now().date_naive()`
  - Parse provided date using `chrono::NaiveDate::parse_from_str`

- **Error Handling**:
  - Map validation errors to gRPC `Status::InvalidArgument`
  - Map DAO errors to appropriate gRPC status codes
  - Log all errors using `tracing` crate
  - Handle MongoDB connection issues as `Status::Unavailable`

### File: `price/src/price-service/handlers/mod.rs`
- Add `pub mod get_best_offer_price;`
- Add handler to service implementation

## 3. Data Access Layer

### File: `price/src/price-service/persistence/offer_dao.rs`
- Add new method `find_best_offer_price`:
  ```rust
  pub async fn find_best_offer_price(
      &self,
      sku: &str,
      quantity: i32,
      date: NaiveDate,
      currency: &str,
  ) -> Result<Option<Offer>, DaoError>
  ```

- **MongoDB Query Implementation**:
  - Use the query from playground-1.mongodb.js line 20 as template
  - Replace hardcoded values with parameters:
    - `sku: { $eq: sku }`
    - `min_quantity: { $lte: quantity }`
    - `max_quantity: { $gte: quantity }`
    - `start_date: { $lte: date }`
    - `end_date: { $gte: date }`
    - `offer_prices: { $elemMatch: { currency: currency } }`
  - Apply sort and limit: `.sort({ "offer_prices.price": 1 }).limit(1)`

- **Error Handling**:
  - Wrap MongoDB errors in custom `DaoError` enum
  - Handle collection access errors
  - Handle deserialization errors
  - Log errors at appropriate levels

## 4. Error Types Enhancement

### File: `price/src/price-service/persistence/mod.rs` or separate error file
- Extend `DaoError` enum:
  ```rust
  #[derive(Debug, thiserror::Error)]
  pub enum DaoError {
      #[error("Database connection error: {0}")]
      ConnectionError(#[from] mongodb::error::Error),
      #[error("Document not found")]
      NotFound,
      #[error("Serialization error: {0}")]
      SerializationError(#[from] bson::ser::Error),
      #[error("Deserialization error: {0}")]
      DeserializationError(#[from] bson::de::Error),
  }
  ```

## 5. Integration Tests Structure

### Directory: `price/tests/`
- `integration_test.rs` - Test module setup and utilities
- `offer_lifecycle_tests.rs` - Tests for create, get, delete operations
- `get_best_offer_price_tests.rs` - Tests for new API

### File: `price/tests/get_best_offer_price_tests.rs`
**Test Cases**:
1. **Happy Path**: Valid request returns lowest priced offer
2. **No Results**: Valid request but no matching offers
3. **Default Date**: Request without date uses current date
4. **Validation Errors**:
   - Empty SKU
   - Negative quantity
   - Invalid currency
   - Invalid date format
5. **Edge Cases**:
   - Multiple offers with same price (consistent ordering)
   - Date boundary conditions
   - Quantity boundary conditions
6. **Error Scenarios**:
   - MongoDB connection failure simulation
   - Collection access errors

### Files for Existing API Tests:
- `price/tests/offer_create_tests.rs`
- `price/tests/offer_get_tests.rs`
- `price/tests/offer_delete_tests.rs`

## 6. Test Infrastructure Setup

### File: `price/tests/common/mod.rs`
- MongoDB test container setup using `testcontainers`
- Test data fixtures and builders
- gRPC client setup utilities
- Cleanup helpers

### File: `price/Cargo.toml` (test dependencies)
```toml
[dev-dependencies]
testcontainers = "0.15"
tokio-test = "0.4"
assert_matches = "1.5"
```

## 7. Logging Integration

- Use `tracing` crate throughout
- Add structured logging with correlation IDs
- Log request parameters (sanitized)
- Log query execution times
- Log error details at appropriate levels

## 8. Implementation Order

1. **Phase 1**: Update protobuf schema and regenerate code
2. **Phase 2**: Implement DAO method with MongoDB query
3. **Phase 3**: Implement handler with validation and error handling
4. **Phase 4**: Set up integration test infrastructure
5. **Phase 5**: Write comprehensive tests for new API
6. **Phase 6**: Add integration tests for existing APIs
7. **Phase 7**: Performance testing and optimization

## 9. Additional Considerations

- **Performance**: Consider indexing strategy for MongoDB queries
- **Caching**: Future consideration for frequently accessed data
- **Monitoring**: Add metrics for API response times and error rates
- **Documentation**: Update API documentation and examples
- **Backwards Compatibility**: Ensure existing APIs remain functional

This plan follows Rust best practices with proper error handling, comprehensive testing, and maintainable code structure suitable for a production MongoDB-backed microservice.

## MongoDB Query Reference

Based on playground-1.mongodb.js line 20:
```javascript
db.getCollection('prices').find({ 
 sku: { $eq: '0096234303' },
 min_quantity: { $lte: 21 },
 max_quantity: { $gte: 21 },
 start_date: { $lte: ISODate('2025-07-26') },
 end_date: { $gte: ISODate('2025-07-26') },
 offer_prices: { $elemMatch: { currency: 'USD' } }
}).sort({ "offer_prices.price": 1 }).limit(1);
```

This query will be parameterized in the DAO implementation.
