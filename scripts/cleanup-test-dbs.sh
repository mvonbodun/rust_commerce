#!/bin/bash

# Script to clean up test databases from MongoDB
# Deletes all databases that start with "test_"

set -e

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# MongoDB connection URL (can be overridden by environment variable)
MONGODB_URL=${MONGODB_URL:-"mongodb://admin:password123@localhost:27017"}

echo -e "${YELLOW}🧹 MongoDB Test Database Cleanup Script${NC}"
echo -e "${YELLOW}=================================${NC}"
echo ""

# Extract just the connection part for display (hide password)
DISPLAY_URL=$(echo "$MONGODB_URL" | sed 's/:\/\/[^:]*:[^@]*@/:\/\/***:***@/')
echo -e "Connecting to: ${DISPLAY_URL}"
echo ""

# Get list of databases starting with "test_"
echo -e "${YELLOW}Searching for test databases...${NC}"

# Use mongosh to list and delete test databases
CLEANUP_SCRIPT='
db.adminCommand({ listDatabases: 1 }).databases.forEach(function(database) {
    if (database.name.startsWith("test_")) {
        print("Found test database: " + database.name);
        db.getSiblingDB(database.name).dropDatabase();
        print("  ✅ Deleted: " + database.name);
    }
});
'

# Count test databases before deletion
COUNT_SCRIPT='
var count = 0;
db.adminCommand({ listDatabases: 1 }).databases.forEach(function(database) {
    if (database.name.startsWith("test_")) {
        count++;
    }
});
print(count);
'

# Get count of test databases
TEST_DB_COUNT=$(mongosh "$MONGODB_URL" --quiet --eval "$COUNT_SCRIPT" 2>/dev/null | tail -1)

if [ "$TEST_DB_COUNT" = "0" ] || [ -z "$TEST_DB_COUNT" ]; then
    echo -e "${GREEN}✨ No test databases found. MongoDB is clean!${NC}"
    exit 0
fi

echo -e "${YELLOW}Found $TEST_DB_COUNT test database(s)${NC}"
echo ""

# Ask for confirmation
read -p "Do you want to delete all test databases? (y/N): " -n 1 -r
echo ""

if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo -e "${YELLOW}Cleanup cancelled${NC}"
    exit 0
fi

echo ""
echo -e "${YELLOW}Cleaning up test databases...${NC}"

# Execute cleanup
mongosh "$MONGODB_URL" --quiet --eval "$CLEANUP_SCRIPT" 2>/dev/null | while IFS= read -r line; do
    if [[ $line == *"Found test database:"* ]]; then
        echo -e "${YELLOW}$line${NC}"
    elif [[ $line == *"✅ Deleted:"* ]]; then
        echo -e "${GREEN}$line${NC}"
    else
        echo "$line"
    fi
done

echo ""
echo -e "${GREEN}✅ Cleanup complete!${NC}"

# Show remaining databases
echo ""
echo -e "${YELLOW}Remaining databases:${NC}"
mongosh "$MONGODB_URL" --quiet --eval "db.adminCommand({ listDatabases: 1 }).databases.forEach(function(d) { print('  - ' + d.name); });" 2>/dev/null