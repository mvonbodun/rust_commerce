#!/bin/bash

# Test script for catalog export functionality
# This script tests the export command with a small sample

echo "🧪 Testing Catalog Export Functionality"
echo "========================================"

# Check if output directory exists, create if not
EXPORT_DIR="./exports"
mkdir -p "$EXPORT_DIR"

# Test export with small batch size
EXPORT_FILE="$EXPORT_DIR/test_export_$(date +%Y%m%d_%H%M%S).json"

echo "📤 Testing export command..."
echo "   File: $EXPORT_FILE"
echo "   Batch Size: 50"
echo ""

# Run the export command
cargo run --bin catalog-client -- export --file "$EXPORT_FILE" --batch-size 50

# Check if export was successful
if [ $? -eq 0 ]; then
    echo ""
    echo "✅ Export command completed successfully!"
    
    # Check if file exists and has content
    if [ -f "$EXPORT_FILE" ]; then
        FILE_SIZE=$(wc -c < "$EXPORT_FILE")
        PRODUCT_COUNT=$(jq '. | length' "$EXPORT_FILE" 2>/dev/null || echo "Could not parse JSON")
        
        echo "📊 Export Results:"
        echo "   File size: $FILE_SIZE bytes"
        echo "   Products exported: $PRODUCT_COUNT"
        echo "   Location: $EXPORT_FILE"
        
        # Show first product as example (if jq is available)
        if command -v jq >/dev/null 2>&1; then
            echo ""
            echo "📋 Sample product (first record):"
            jq '.[0] | {name, product_ref, brand, display_on_site}' "$EXPORT_FILE" 2>/dev/null || echo "   Could not display sample"
        fi
    else
        echo "❌ Export file not found!"
        exit 1
    fi
else
    echo ""
    echo "❌ Export command failed!"
    echo ""
    echo "🔍 Troubleshooting checklist:"
    echo "   □ Is NATS server running? (nats-server)"
    echo "   □ Is MongoDB running?"
    echo "   □ Is catalog-service running? (cargo run --bin catalog-service)"
    echo "   □ Is MONGODB_URL environment variable set?"
    echo ""
    exit 1
fi

echo ""
echo "🎉 Export test completed successfully!"
