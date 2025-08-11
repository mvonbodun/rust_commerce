// MongoDB Category Collection Indexes
// Run this script in MongoDB shell or MongoDB Compass

// Switch to your catalog database
// use catalog;

// Primary category collection indexes
db.categories.createIndex({ "slug": 1 }, { unique: true });
db.categories.createIndex({ "path": 1 });
db.categories.createIndex({ "parent_id": 1 });
db.categories.createIndex({ "ancestors": 1 });
db.categories.createIndex({ "level": 1 });
db.categories.createIndex({ "is_active": 1, "display_order": 1 });
db.categories.createIndex({ "parent_id": 1, "is_active": 1, "display_order": 1 });

// Text search index for category search functionality
db.categories.createIndex({ 
    "name": "text", 
    "short_description": "text", 
    "seo.keywords": "text" 
}, {
    name: "category_text_search",
    default_language: "english"
});

// Cache collection index
db.category_tree_cache.createIndex({ "version": 1 });
db.category_tree_cache.createIndex({ "last_updated": 1 });

// Display created indexes
print("Created indexes for categories collection:");
db.categories.getIndexes().forEach(function(index) {
    print("  - " + index.name + ": " + JSON.stringify(index.key));
});

print("\nCreated indexes for category_tree_cache collection:");
db.category_tree_cache.getIndexes().forEach(function(index) {
    print("  - " + index.name + ": " + JSON.stringify(index.key));
});
