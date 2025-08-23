#!/bin/bash

# This script checks if the generated TypeScript files are in sync with proto files
# Run this before committing or in CI to ensure consistency

set -e

echo "🔍 Checking if generated files are in sync with proto files..."

# Save current generated files
if [ -d "generated" ]; then
    mv generated generated.backup
fi

# Regenerate from proto
echo "📝 Regenerating TypeScript from proto files..."
npm run generate:proto > /dev/null 2>&1
npm run generate:nats-config > /dev/null 2>&1

# Check for differences
if [ -d "generated.backup" ]; then
    if diff -r generated.backup generated > /dev/null 2>&1; then
        echo "✅ Generated files are in sync with proto files"
        rm -rf generated.backup
        exit 0
    else
        echo "❌ Generated files are out of sync with proto files!"
        echo "   Please run 'npm run generate' and commit the changes."
        echo ""
        echo "Differences found:"
        diff -r generated.backup generated || true
        
        # Restore backup
        rm -rf generated
        mv generated.backup generated
        exit 1
    fi
else
    echo "⚠️  No existing generated files to compare"
    echo "   Generated files have been created."
    exit 0
fi