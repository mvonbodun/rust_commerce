#!/bin/bash

# Script to run catalog tests properly
# Tests need to run sequentially to avoid NATS subject conflicts

set -e

# Color codes for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${YELLOW}🧪 Running Catalog Integration Tests${NC}"
echo -e "${YELLOW}===================================${NC}"
echo ""

# Check if specific test pattern was provided
if [ $# -eq 0 ]; then
    echo -e "${YELLOW}Running all catalog tests sequentially...${NC}"
    TEST_PATTERN=""
else
    echo -e "${YELLOW}Running tests matching: $1${NC}"
    TEST_PATTERN="$1"
fi

echo ""

# Run tests sequentially to avoid conflicts
# Each test spawns its own catalog service instance that listens on catalog.*
# Running in parallel causes NATS message routing conflicts
if cargo test --test api $TEST_PATTERN -- --test-threads=1 --nocapture 2>&1 | tee test-output.log; then
    echo ""
    echo -e "${GREEN}✅ All tests passed!${NC}"
    
    # Clean up test databases
    echo ""
    echo -e "${YELLOW}Cleaning up test databases...${NC}"
    ./scripts/cleanup-test-dbs-auto.sh 2>/dev/null || true
    
    exit 0
else
    echo ""
    echo -e "${RED}❌ Some tests failed. Check test-output.log for details${NC}"
    exit 1
fi