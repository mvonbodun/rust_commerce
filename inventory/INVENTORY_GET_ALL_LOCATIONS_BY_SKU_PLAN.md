# InventoryGetAllLocationsBySku API Implementation Plan

## Overview
This plan outlines the implementation of a new API endpoint `InventoryGetAllLocationsBySku` that accepts multiple SKUs and returns aggregated inventory data across all locations for each SKU, along with detailed location-specific breakdowns.

## Current State Analysis

### Existing Inventory Structure
- **Single Location per Record**: Each MongoDB document represents inventory for one SKU at one specific location
- **Location Examples**: "DC - Dallas, TX", "STORE - Spring, TX", "STORE - Austin, TX"
- **Current API**: `InventoryGetRequest` only handles single SKU queries
- **Data Fields**: quantity, reserved_quantity, available_quantity, min_stock_level, location

### Business Requirements
1. **Multi-SKU Query**: Accept an array of SKUs in a single request
2. **Aggregated Summary**: For each SKU, provide totals across all locations
3. **Location Breakdown**: Include detailed inventory for each location per SKU
4. **Performance**: Efficient querying to minimize database round trips

## Proposed Solution

### 1. Protocol Buffer Definitions

#### New Request Message
```protobuf
message InventoryGetAllLocationsBySkuRequest {
    repeated string skus = 1;
}
```

#### New Response Messages
```protobuf
message InventoryGetAllLocationsBySkuResponse {
    repeated SkuInventorySummary sku_summaries = 1;
    Status status = 2;
}

message SkuInventorySummary {
    string sku = 1;
    InventoryAggregation total_inventory = 2;
    repeated InventoryLocationDetail location_details = 3;
}

message InventoryAggregation {
    int32 total_quantity = 1;
    int32 total_reserved_quantity = 2;
    int32 total_available_quantity = 3;
    int32 min_stock_level_across_locations = 4;
    int32 location_count = 5;
}

message InventoryLocationDetail {
    string location = 1;
    int32 quantity = 2;
    int32 reserved_quantity = 3;
    int32 available_quantity = 4;
    int32 min_stock_level = 5;
    google.protobuf.Timestamp last_updated = 6;
    google.protobuf.Timestamp created_at = 7;
}
```

### 2. Database Layer Implementation

#### DAO Method Addition
- **Method**: `get_inventory_by_skus(&self, skus: Vec<String>) -> Result<HashMap<String, Vec<InventoryItem>>, DBError>`
- **Query Strategy**: Use MongoDB aggregation pipeline or `$in` operator for efficient multi-SKU retrieval
- **Data Structure**: Return HashMap with SKU as key and Vec of InventoryItem (one per location) as value

#### MongoDB Query Optimization
```javascript
// Example aggregation pipeline
db.inventory.aggregate([
  { $match: { sku: { $in: ["SKU1", "SKU2", "SKU3"] } } },
  { $sort: { sku: 1, location: 1 } }
])
```

### 3. Service Layer Implementation

#### Handler Function
- **Name**: `get_inventory_all_locations_by_sku`
- **Location**: `inventory/src/inventory-service/handlers/mod.rs`
- **Responsibilities**:
  1. Validate input SKUs (non-empty array, valid SKU format)
  2. Call DAO method to retrieve inventory data
  3. Aggregate data per SKU across locations
  4. Build response with summaries and details

#### Business Logic
```rust
// Aggregation logic for each SKU
fn aggregate_inventory_for_sku(items: &[InventoryItem]) -> InventoryAggregation {
    InventoryAggregation {
        total_quantity: items.iter().map(|i| i.quantity).sum(),
        total_reserved_quantity: items.iter().map(|i| i.reserved_quantity).sum(),
        total_available_quantity: items.iter().map(|i| i.available_quantity).sum(),
        min_stock_level_across_locations: items.iter().map(|i| i.min_stock_level).min().unwrap_or(0),
        location_count: items.len() as i32,
    }
}
```

### 4. NATS Route Registration

#### Route Addition
- **Subject**: `inventory.get_all_locations_by_sku`
- **Handler**: `get_inventory_all_locations_by_sku`
- **Location**: `inventory/src/inventory-service/main.rs`

### 5. Client Implementation

#### CLI Command Addition
- **Command**: `inventory-get-multi-sku`
- **Parameters**: `--skus <SKU1,SKU2,SKU3>` (comma-separated list)
- **Output**: Formatted display of aggregated and detailed inventory data

#### Example CLI Output
```
SKU: WIDGET-001
  Total Inventory: 150 units (120 available, 30 reserved)
  Locations: 3
  ├─ DC - Dallas, TX: 100 units (80 available, 20 reserved)
  ├─ STORE - Spring, TX: 30 units (25 available, 5 reserved)
  └─ STORE - Austin, TX: 20 units (15 available, 5 reserved)

SKU: GADGET-002
  Total Inventory: 75 units (60 available, 15 reserved)
  Locations: 2
  ├─ DC - Dallas, TX: 50 units (40 available, 10 reserved)
  └─ STORE - Spring, TX: 25 units (20 available, 5 reserved)
```

### 6. Integration Tests

#### Test Scenarios
1. **Multi-SKU Success**: Query multiple existing SKUs across different locations
2. **Partial Results**: Some SKUs exist, others don't
3. **Empty SKU List**: Handle empty input array
4. **Invalid SKUs**: Handle non-existent SKUs
5. **Single Location**: SKU exists in only one location
6. **Performance**: Test with large SKU lists (50+ SKUs)

#### Test Data Setup
- Create test inventory items for 3-4 SKUs across 3-4 locations
- Include edge cases (SKUs with no inventory, single location SKUs)

### 7. Error Handling

#### Validation Errors
- Empty SKU array
- Invalid SKU format
- SKU array too large (implement reasonable limit, e.g., 100 SKUs per request)

#### Database Errors
- Connection failures
- Query timeouts
- Data consistency issues

#### Response Codes
- `OK`: Successful retrieval with data
- `NOT_FOUND`: None of the requested SKUs exist
- `PARTIAL_SUCCESS`: Some SKUs found, others not (include details in response)
- `INVALID_REQUEST`: Validation errors
- `INTERNAL_ERROR`: Database or system errors

### 8. Performance Considerations

#### Database Optimization
- **Indexing**: Ensure compound index on `(sku, location)` for efficient querying
- **Connection Pooling**: Reuse database connections
- **Query Batching**: Single query for all SKUs rather than N+1 queries

#### Memory Management
- **Streaming**: For very large result sets, consider streaming responses
- **Caching**: Optional caching layer for frequently requested SKU combinations
- **Pagination**: Consider pagination for responses with many locations per SKU

### 9. Implementation Steps

#### Phase 1: Proto and Model Updates
1. Update `inventory.proto` with new message definitions
2. Regenerate protobuf code
3. Update Rust model structures if needed

#### Phase 2: Database Layer
1. Add DAO method for multi-SKU queries
2. Implement efficient MongoDB aggregation
3. Add unit tests for DAO layer

#### Phase 3: Service Layer
1. Implement handler function with aggregation logic
2. Add input validation
3. Register NATS route

#### Phase 4: Client and Testing
1. Add CLI command for multi-SKU queries
2. Implement integration tests
3. Performance testing with various SKU counts

#### Phase 5: Documentation and Optimization
1. Update API documentation
2. Performance tuning based on test results
3. Add monitoring and logging

### 10. Success Criteria

#### Functional Requirements
- ✅ Accept array of SKUs in single request
- ✅ Return aggregated totals for each SKU
- ✅ Include location-specific breakdowns
- ✅ Handle edge cases (missing SKUs, single locations)
- ✅ Proper error handling and validation

#### Performance Requirements
- ✅ Response time < 500ms for 10 SKUs
- ✅ Response time < 2s for 50 SKUs
- ✅ Support concurrent requests without degradation
- ✅ Efficient database querying (single query per request)

#### Quality Requirements
- ✅ Comprehensive test coverage (>80%)
- ✅ Clear error messages and status codes
- ✅ Consistent API design with existing endpoints
- ✅ Proper logging and monitoring

## Next Steps

1. **Review and Approval**: Review this plan and provide feedback
2. **Technical Validation**: Validate MongoDB query performance with existing data
3. **Implementation**: Begin with Phase 1 (Proto and Model Updates)
4. **Iterative Development**: Implement and test each phase incrementally
5. **Production Deployment**: Deploy with feature flags for gradual rollout

## Questions for Consideration

1. **SKU Limit**: What's the maximum number of SKUs we should accept in a single request?
2. **Response Size**: Should we implement pagination for SKUs with many locations?
3. **Caching**: Do we need caching for frequently requested SKU combinations?
4. **Backwards Compatibility**: Any concerns about proto changes affecting existing clients?
5. **Authentication**: Does this endpoint need special authentication/authorization?

---

*This plan provides a comprehensive roadmap for implementing the InventoryGetAllLocationsBySku API while maintaining performance, reliability, and consistency with the existing codebase.*
