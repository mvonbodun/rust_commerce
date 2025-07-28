# Catalog Export Functionality

This document describes the new export functionality added to the catalog service.

## New Features

### 1. Export All Products
- **Command**: `catalog.export_products`
- **Description**: Exports all products from the MongoDB database
- **Batch Processing**: Uses cursor-based pagination to handle large datasets efficiently
- **Default Batch Size**: 1000 products per batch (configurable)

### 2. CLI Export Command
- **Usage**: `cargo run --bin catalog-client -- export --file <FILE> [--batch-size <SIZE>]`
- **Parameters**:
  - `--file`: Required. Path to the output JSON file
  - `--batch-size`: Optional. Number of products to fetch per batch (default: 1000)

## Usage Examples

```bash
# Export all products to a file with default batch size
cargo run --bin catalog-client -- export --file products_export.json

# Export with custom batch size
cargo run --bin catalog-client -- export --file products_export.json --batch-size 500

# Export to a specific directory
cargo run --bin catalog-client -- export --file /path/to/exports/products_$(date +%Y%m%d).json
```

## Prerequisites

1. **NATS Server**: Must be running on `0.0.0.0:4222`
   ```bash
   # Install and run NATS server
   nats-server
   ```

2. **MongoDB**: Must be running with the catalog database
   - Database: `db_catalog`
   - Collection: `products`
   - Connection string in `MONGODB_URL` environment variable

3. **Catalog Service**: Must be running
   ```bash
   cd catalog
   cargo run --bin catalog-service
   ```

## Output Format

The export generates a JSON file containing an array of product objects with all fields:

```json
[
  {
    "id": "product_id_here",
    "name": "Product Name",
    "product_ref": "PROD-001",
    "brand": "Brand Name",
    "long_description": "Detailed description...",
    "slug": "product-name-prod-001",
    "seo_title": "SEO Title",
    "seo_description": "SEO Description",
    "seo_keywords": "keywords, product",
    "display_on_site": true,
    "tax_code": "txcd_99999999",
    "list_categories": ["Category 1", "Category 2"],
    "defining_attributes": {},
    "descriptive_attributes": {},
    "variants": [],
    "created_at": "2025-01-01T00:00:00Z",
    "updated_at": "2025-01-01T00:00:00Z"
  }
]
```

## Implementation Details

### Service Layer
- **New Handler**: `export_products` in `handlers/mod.rs`
- **Protocol**: Added `ProductExportRequest` and `ProductExportResponse` messages
- **Route**: `catalog.export_products`

### Data Access Layer
- **New Method**: `export_all_products` in `ProductDao` trait
- **Pagination**: Uses MongoDB cursor with `skip` and `limit` for efficient batch processing
- **Memory Management**: Processes data in configurable batches to avoid memory issues

### Client Layer
- **New Command**: `export` command in catalog-client
- **Conversion**: Automatic conversion from protobuf messages to domain models
- **File Output**: Pretty-printed JSON format

## Performance Considerations

1. **Batch Size**: Adjust `--batch-size` based on available memory and network capacity
2. **Large Datasets**: The cursor-based approach handles millions of records efficiently
3. **Network Timeout**: For very large exports, consider increasing NATS timeout settings
4. **Memory Usage**: Each batch is processed and released, keeping memory usage constant

## Error Handling

- **Connection Errors**: Client will report NATS connection issues
- **Database Errors**: Service logs MongoDB errors and returns appropriate status codes
- **File Errors**: Client reports file system write errors
- **Timeout Errors**: Increase batch size or check service availability

## Future Enhancements

1. **Streaming Export**: For extremely large datasets, implement streaming to file
2. **Filtered Export**: Add query parameters to export specific product subsets
3. **Format Options**: Support for CSV, XML, or other export formats
4. **Compression**: Add gzip compression for large export files
5. **Resume Capability**: Support for resuming interrupted exports
