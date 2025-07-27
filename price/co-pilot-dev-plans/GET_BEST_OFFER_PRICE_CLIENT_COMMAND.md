# GetBestOfferPrice Command Added to Price Client

## Summary

Successfully added the `GetBestOfferPrice` command to the price-client CLI application.

## New Command Usage

```bash
./target/debug/price-client get-best-offer-price --sku <SKU> --quantity <QUANTITY> [OPTIONS]
```

### Parameters

- `--sku, -s <SKU>` (required): The product SKU to search for
- `--quantity, -q <QUANTITY>` (required): The quantity of items needed
- `--currency, -c <CURRENCY>` (optional): Currency code (default: USD)
- `--date, -d <DATE>` (optional): Date in ISO 8601 format (defaults to current date)

### Examples

```bash
# Get best offer for 5 units of product ABC123 in USD
./target/debug/price-client get-best-offer-price --sku ABC123 --quantity 5

# Get best offer with specific currency
./target/debug/price-client get-best-offer-price --sku ABC123 --quantity 10 --currency EUR

# Get best offer for a specific date
./target/debug/price-client get-best-offer-price --sku ABC123 --quantity 5 --date 2025-07-26T00:00:00Z
```

### Response Format

**When offer is found:**
```
✅ Found best offer:
  Offer ID: 64f1a2b3c4d5e6f7g8h9i0j1
  SKU: ABC123
  Min Quantity: 1
  Max Quantity: 100
  Prices:
    - 19.99 USD
    - 24.99 EUR
```

**When no offer is found:**
```
❌ No offer found for SKU: ABC123 with quantity: 5 in currency: USD
```

## Technical Implementation

- Added `GetBestOfferPriceRequest` and `GetBestOfferPriceResponse` imports
- Added new `GetBestOfferPrice` command to the CLI enum
- Implemented request handling with proper validation
- Added formatted output for found offers
- Sends request to NATS subject: `offers.get_best_offer_price`

## Integration

This command integrates with the existing GetBestOfferPrice API that was implemented in the price service handlers. The complete request/response flow is now available through the CLI client.
