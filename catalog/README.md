# Catalog Service

The catalog service manages product information in the rust_commerce microservices architecture.

## Status

🚧 **Work in Progress** - This service scaffolding has been created but handlers are not yet implemented.

## Features (Planned)

- **Product Management**: Create, read, update, and delete products
- **Product Search**: Search products by name, description, category, and brand
- **Variant Support**: Handle product variants with different attributes (size, color, etc.)
- **Category Management**: Hierarchical category structure support
- **Inventory Tracking**: Quantity tracking per variant
- **Pricing Information**: Multiple price points (list, sale, MSRP)
- **SEO Support**: SEO-friendly titles, descriptions, and keywords

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

- **Basic Info**: name, description, brand, product reference
- **SEO**: title, description, keywords, slug
- **Categories**: hierarchical and flat category structures
- **Attributes**: defining attributes (size, color) and descriptive attributes
- **Variants**: multiple SKUs per product with individual pricing and inventory
- **Reviews**: aggregate review data
- **Metadata**: created/updated timestamps and user tracking

### Variants

Each product can have multiple variants with:

- **SKU**: unique stock keeping unit
- **Attributes**: defining characteristics (size, color, etc.)
- **Inventory**: quantity available and reserved
- **Pricing**: list price, sale price, MSRP
- **Images**: variant-specific images

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

## Next Steps

1. Implement handler business logic
2. Add data model conversions between protobuf and domain models
3. Complete DAO implementations
4. Add validation and error handling
5. Implement full product lifecycle operations
6. Add comprehensive tests
