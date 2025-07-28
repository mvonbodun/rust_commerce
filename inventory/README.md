# Inventory Service

The inventory service manages product inventory levels, stock tracking, and availability for the rust_commerce platform.

## Features

- Track inventory levels for products
- Manage stock adjustments
- Monitor low stock alerts
- Handle inventory reservations

## API

The service exposes the following operations via NATS:
- `inventory.create_item` - Create new inventory item
- `inventory.get_item` - Get inventory item by SKU
- `inventory.update_stock` - Update stock levels
- `inventory.delete_item` - Delete inventory item

## Setup

1. Set the `MONGODB_URL` environment variable
2. Ensure NATS server is running on `0.0.0.0:4222`
3. Run the service: `cargo run --bin inventory-service`

## Client

Use the inventory client to interact with the service:
```bash
cargo run --bin inventory-client -- --help
```
