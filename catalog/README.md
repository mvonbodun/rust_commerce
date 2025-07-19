# Catalog Service

The catalog service manages product information in the rust_commerce microservices architecture.

## Status

✅ **Implemented** - Core handlers and CLI client are complete and functional.

**Completed:**
- NATS message handlers for all CRUD operations
- Domain model with builder pattern and UUID generation
- Protocol Buffer message conversion
- CLI client with comprehensive command support
- Error handling and response formatting

**Next Steps:**
- Database integration testing
- Integration tests for NATS messaging
- Production deployment configuration

## Features

- ✅ **Product Management**: Create, read, update, and delete products
- ✅ **Product Search**: Search products by name, description, category, and brand
- ✅ **Variant Support**: Handle product variants with different attributes (size, color, etc.)
- ✅ **Category Management**: Hierarchical category structure support
- ✅ **Builder Pattern**: Fluent API for constructing domain models
- ✅ **UUID Generation**: Automatic ID generation for new products
- ✅ **Protocol Buffer Integration**: Full message encoding/decoding
- ✅ **CLI Client**: Command-line interface for testing and administration
- 🚧 **Inventory Tracking**: Quantity tracking per variant (planned)
- 🚧 **Pricing Information**: Multiple price points (list, sale, MSRP) (planned)
- ✅ **SEO Support**: SEO-friendly titles, descriptions, and keywords

## API Operations

### Via NATS Messaging

- `catalog.create_product` - Create a new product
- `catalog.get_product` - Retrieve product by ID
- `catalog.update_product` - Update existing product
- `catalog.delete_product` - Delete product
- `catalog.search_products` - Search products with filters

## Data Model

### Product

The core product entity includes:

- **Basic Info**: name, description, brand, numeric product reference
- **SEO**: title, description, keywords, slug
- **Categories**: hierarchical and flat category structures  
- **Attributes**: defining attributes (size, color) and descriptive attributes
- **Variants**: multiple SKUs per product with dimensions, weight, and packaging info
- **Reviews**: aggregate review data
- **Metadata**: created/updated timestamps and user tracking

### Variants

Each product can have multiple variants with:

- **SKU**: unique stock keeping unit
- **Attributes**: defining characteristics (size, color, etc.)
- **Dimensions**: height, width, length, weight with units
- **Packaging**: separate packaging dimensions and weight
- **Images**: array of image URLs for the variant
- **Abbreviated Info**: shortened color and size descriptions

## Database

- **Database**: `db_catalog`
- **Collection**: `products`
- **Schema**: Flexible document structure supporting complex product hierarchies

## Building and Running

```bash
# Build the service
cargo build -p rust-catalog

# Run the service
cargo run --bin catalog-service

# Run the test client
cargo run --bin catalog-client -- --help
```

## Testing

```bash
# Test product creation
cargo run --bin catalog-client -- product-create --name "Sample Product" --brand "Sample Brand"

# Test product search
cargo run --bin catalog-client -- product-search --query "sample"
```

## Catalog Client CLI

The catalog-client is a command-line interface for interacting with the catalog service via NATS messaging.

### Prerequisites

- NATS server running on `0.0.0.0:4222`
- Catalog service running and subscribed to catalog topics

### Available Commands

#### Product Create

Creates a new product in the catalog.

```bash
cargo run --bin catalog-client -- product-create --name <NAME> --product-ref <PRODUCT_REF> [--brand <BRAND>]
```

**Required Arguments:**
- `--name, -n <NAME>`: The name of the product
- `--product-ref, -p <PRODUCT_REF>`: The product reference/SKU

**Optional Arguments:**
- `--brand, -b <BRAND>`: The brand of the product

**Examples:**
```bash
# Create a product with required fields
cargo run --bin catalog-client -- product-create --name "iPhone 15" --product-ref "IPH15001"

# Create a product with name, reference, and brand
cargo run --bin catalog-client -- product-create --name "iPhone 15" --product-ref "IPH15001" --brand "Apple"
```

**Default Values:** The client automatically sets sample values for description, SEO fields, categories, and tax codes.

#### Product Get

Retrieves a product by its ID.

```bash
cargo run --bin catalog-client -- product-get --id <ID>
```

**Required Arguments:**
- `--id, -i <ID>`: The unique identifier of the product

**Example:**
```bash
cargo run --bin catalog-client -- product-get --id "507f1f77bcf86cd799439011"
```

#### Product Delete

Deletes a product by its ID.

```bash
cargo run --bin catalog-client -- product-delete --id <ID>
```

**Required Arguments:**
- `--id, -i <ID>`: The unique identifier of the product to delete

**Example:**
```bash
cargo run --bin catalog-client -- product-delete --id "507f1f77bcf86cd799439011"
```

#### Product Search

Searches for products based on various criteria.

```bash
cargo run --bin catalog-client -- product-search [--query <QUERY>] [--category <CATEGORY>] [--brand <BRAND>]
```

**Optional Arguments:**
- `--query, -q <QUERY>`: Search query text to match against product names and descriptions
- `--category, -c <CATEGORY>`: Filter by category name
- `--brand, -b <BRAND>`: Filter by brand name

**Examples:**
```bash
# Search all products (returns up to 10 products)
cargo run --bin catalog-client -- product-search

# Search by query text
cargo run --bin catalog-client -- product-search --query "iPhone"

# Search by brand
cargo run --bin catalog-client -- product-search --brand "Apple"

# Combined search
cargo run --bin catalog-client -- product-search --query "phone" --brand "Apple" --category "Electronics"
```

#### Import

Imports products from a JSON file containing product data. Supports both single product objects and arrays of products.

```bash
cargo run --bin catalog-client -- import --file <FILE> [--dry-run]
```

**Required Arguments:**
- `--file, -f <FILE>`: Path to the JSON file containing product data

**Optional Arguments:**
- `--dry-run, -d`: Preview what would be imported without actually creating products

**Examples:**
```bash
# Preview import without creating products
cargo run --bin catalog-client -- import --file sample_records_backup/sample_product_mongo_record.json --dry-run

# Import products from a JSON file
cargo run --bin catalog-client -- import --file products.json

# Import a single product from JSON
cargo run --bin catalog-client -- import --file single_product.json
```

**JSON Format:**
The import command expects JSON files matching the product model structure. It supports:
- Single product objects (like `sample_product_mongo_record.json`)
- Arrays of product objects
- All product fields including variants, categories, reviews, and attributes

**Import Features:**
- Automatic detection of single products vs arrays
- Progress reporting during import
- Error handling for individual product failures
- Success/failure summary
- Rate limiting (100ms delay between products) to avoid overwhelming the service
- Comprehensive status reporting with success/error counts

### Response Format

All commands return Protocol Buffer responses that include:
- **Success responses**: Product data with status code and success message
- **Error responses**: Status code, error message, and details

### Example Workflow

```bash
# 1. Import products from a JSON file
cargo run --bin catalog-client -- import --file sample_records_backup/sample_product_mongo_record.json --dry-run
cargo run --bin catalog-client -- import --file sample_records_backup/sample_product_mongo_record.json

# 2. Create a new product manually
cargo run --bin catalog-client -- product-create --name "MacBook Pro" --product-ref "MBP001" --brand "Apple"
# Note the ID from the response

# 3. Retrieve the product by ID
cargo run --bin catalog-client -- product-get --id "PRODUCT_ID_FROM_STEP_2"

# 4. Search for products by brand
cargo run --bin catalog-client -- product-search --brand "Apple"

# 5. Search for imported products
cargo run --bin catalog-client -- product-search --brand "Calvin Klein Performance"

# 6. Delete a product
cargo run --bin catalog-client -- product-delete --id "PRODUCT_ID_FROM_STEP_2"
```

### Help

To see all available commands:
```bash
cargo run --bin catalog-client -- --help
```

To see help for a specific command:
```bash
cargo run --bin catalog-client -- product-create --help
```

## Next Steps

1. Implement handler business logic
2. Add data model conversions between protobuf and domain models
3. Complete DAO implementations
4. Add validation and error handling
5. Implement full product lifecycle operations
6. Add comprehensive tests
