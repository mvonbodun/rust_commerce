#!/bin/bash

# Script to automatically clean up test databases from MongoDB (non-interactive)
# Deletes all databases that start with "test_" without prompting

set -e

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# MongoDB connection URL (can be overridden by environment variable)
MONGODB_URL=${MONGODB_URL:-"mongodb://admin:password123@localhost:27017"}

echo -e "${YELLOW}🧹 MongoDB Test Database Cleanup (Automatic)${NC}"
echo -e "${YELLOW}==========================================${NC}"
echo ""

# Extract just the connection part for display (hide password)
DISPLAY_URL=$(echo "$MONGODB_URL" | sed 's/:\/\/[^:]*:[^@]*@/:\/\/***:***@/')
echo -e "Connecting to: ${DISPLAY_URL}"
echo ""

# Get list of databases starting with "test_"
echo -e "${YELLOW}Searching for test databases...${NC}"

# Use mongosh to list and delete test databases
CLEANUP_SCRIPT='
var count = 0;
db.adminCommand({ listDatabases: 1 }).databases.forEach(function(database) {
    if (database.name.startsWith("test_")) {
        print("Deleting: " + database.name);
        db.getSiblingDB(database.name).dropDatabase();
        count++;
    }
});
if (count == 0) {
    print("No test databases found");
} else {
    print("Deleted " + count + " test database(s)");
}
'

# Execute cleanup
OUTPUT=$(mongosh "$MONGODB_URL" --quiet --eval "$CLEANUP_SCRIPT" 2>/dev/null)

# Process and display output
echo "$OUTPUT" | while IFS= read -r line; do
    if [[ $line == *"Deleting:"* ]]; then
        echo -e "${YELLOW}  🗑️  $line${NC}"
    elif [[ $line == *"No test databases found"* ]]; then
        echo -e "${GREEN}✨ $line${NC}"
    elif [[ $line == *"Deleted"*"database"* ]]; then
        echo -e "${GREEN}✅ $line${NC}"
    else
        echo "$line"
    fi
done

echo ""
echo -e "${GREEN}✅ Cleanup complete!${NC}"

# Show remaining databases
echo ""
echo -e "${YELLOW}Remaining databases:${NC}"
mongosh "$MONGODB_URL" --quiet --eval "db.adminCommand({ listDatabases: 1 }).databases.forEach(function(d) { if (!d.name.startsWith('test_')) { print('  - ' + d.name); } });" 2>/dev/null