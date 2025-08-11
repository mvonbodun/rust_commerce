# Category Hierarchy Implementation Plan
## Hybrid Approach for Catalog Module

### Overview
This document outlines the implementation plan for a category hierarchy system using a hybrid materialized path + summary tree approach within the rust_commerce catalog module. The design balances read performance, write flexibility, and maintainability for e-commerce category management.

---

## 1. Data Model Design

### 1.1 Primary Category Collection (`categories`)

**Document Structure:**
```json
{
  "_id": "550e8400-e29b-41d4-a716-446655440000",
  "slug": "smartphones",
  "name": "Smartphones", 
  "short_description": "Mobile phones and accessories",
  "full_description": "Complete range of smartphones from leading brands...",
  "path": "electronics.smartphones",
  "ancestors": ["550e8400-e29b-41d4-a716-446655440001"],
  "parent_id": "550e8400-e29b-41d4-a716-446655440001",
  "level": 1,
  "children_count": 5,
  "product_count": 1250,
  "is_active": true,
  "display_order": 2,
  "seo": {
    "meta_title": "Smartphones - Best Mobile Phones",
    "meta_description": "Browse our extensive collection of smartphones...",
    "keywords": ["smartphones", "mobile", "phones", "cellular"]
  },
  "created_at": "2025-01-01T00:00:00Z",
  "updated_at": "2025-01-01T00:00:00Z"
}
```

**Note:** The `_id` field uses UUID v4 format (String) for consistency with the Product model, and all parent/ancestor references use the same UUID format.

### 1.2 Category Tree Cache Collection (`category_tree_cache`)

**Document Structure:**
```json
{
  "_id": "category_tree_v1",
  "version": 1,
  "last_updated": "2025-01-01T00:00:00Z",
  "tree": {
    "550e8400-e29b-41d4-a716-446655440001": {
      "id": "550e8400-e29b-41d4-a716-446655440001",
      "name": "Electronics",
      "slug": "electronics",
      "level": 0,
      "product_count": 5000,
      "children": {
        "550e8400-e29b-41d4-a716-446655440000": {
          "id": "550e8400-e29b-41d4-a716-446655440000", 
          "name": "Smartphones",
          "slug": "smartphones",
          "level": 1,
          "product_count": 1250,
          "children": { /* ... */ }
        }
      }
    }
  }
}
```

---

## 2. Rust Data Structures

### 2.1 Core Models (`src/model.rs`)

**New Structs to Add:**
```rust
// Category model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Category {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<String>, // UUID v4 format
    pub slug: String,
    pub name: String,
    pub short_description: String,
    pub full_description: Option<String>,
    pub path: String,
    pub ancestors: Vec<String>, // UUIDs of ancestor categories
    pub parent_id: Option<String>, // UUID of parent category
    pub level: i32,
    pub children_count: i32,
    pub product_count: i32,
    pub is_active: bool,
    pub display_order: i32,
    pub seo: CategorySeo,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// SEO metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategorySeo {
    pub meta_title: Option<String>,
    pub meta_description: Option<String>,
    pub keywords: Vec<String>,
}

// Simplified category for tree cache
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryTreeNode {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub level: i32,
    pub product_count: i32,
    pub children: HashMap<String, CategoryTreeNode>,
}

// Tree cache document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryTreeCache {
    #[serde(rename = "_id")]
    pub id: String,
    pub version: i32,
    pub last_updated: DateTime<Utc>,
    pub tree: HashMap<String, CategoryTreeNode>,
}
```

### 2.2 Database Collections

**Add to main.rs or database setup:**
```rust
// Additional collections to initialize
let categories_collection: Collection<Category> = database.collection("categories");
let category_cache_collection: Collection<CategoryTreeCache> = database.collection("category_tree_cache");
```

---

## 3. Data Access Layer

### 3.1 Category DAO Interface (`src/persistence/category_dao.rs`)

**New file to create with trait definition:**
```rust
#[async_trait]
pub trait CategoryDao {
    // CRUD Operations
    async fn create_category(&self, category: Category) -> Result<Category, Box<dyn Error + Send + Sync>>;
    async fn get_category(&self, id: &str) -> Result<Option<Category>, Box<dyn Error + Send + Sync>>;
    async fn get_category_by_slug(&self, slug: &str) -> Result<Option<Category>, Box<dyn Error + Send + Sync>>;
    async fn update_category(&self, id: &str, category: Category) -> Result<Option<Category>, Box<dyn Error + Send + Sync>>;
    async fn delete_category(&self, id: &str) -> Result<bool, Box<dyn Error + Send + Sync>>;
    
    // Hierarchy Operations
    async fn get_children(&self, parent_id: &str) -> Result<Vec<Category>, Box<dyn Error + Send + Sync>>;
    async fn get_descendants(&self, ancestor_id: &str) -> Result<Vec<Category>, Box<dyn Error + Send + Sync>>;
    async fn get_ancestors(&self, category_id: &str) -> Result<Vec<Category>, Box<dyn Error + Send + Sync>>;
    async fn get_breadcrumbs(&self, category_id: &str) -> Result<Vec<Category>, Box<dyn Error + Send + Sync>>;
    async fn move_category(&self, category_id: &str, new_parent_id: Option<&str>) -> Result<bool, Box<dyn Error + Send + Sync>>;
    
    // Tree Cache Operations
    async fn get_full_tree(&self) -> Result<Option<CategoryTreeCache>, Box<dyn Error + Send + Sync>>;
    async fn rebuild_tree_cache(&self) -> Result<CategoryTreeCache, Box<dyn Error + Send + Sync>>;
    async fn invalidate_tree_cache(&self) -> Result<bool, Box<dyn Error + Send + Sync>>;
    
    // Utility Operations
    async fn update_product_counts(&self) -> Result<bool, Box<dyn Error + Send + Sync>>;
    async fn reorder_children(&self, parent_id: &str, ordered_ids: Vec<String>) -> Result<bool, Box<dyn Error + Send + Sync>>;
    
    // Import/Export Operations
    async fn export_all_categories(&self, batch_size: Option<i64>) -> Result<Vec<Category>, Box<dyn Error + Send + Sync>>;
    async fn export_categories_batch(&self, batch_size: Option<i64>, offset: Option<u64>) -> Result<Vec<Category>, Box<dyn Error + Send + Sync>>;
}
```

### 3.2 Implementation Structure

**Key methods to implement:**
- Path generation and validation
- Ancestor array management
- Recursive tree building for cache
- Product count aggregation
- Atomic move operations with path updates

---

## 4. Service Layer

### 4.1 Category Service (`src/handlers/category_service.rs`)

**New file to create with business logic:**
```rust
pub struct CategoryService {
    category_dao: Arc<dyn CategoryDao>,
    product_dao: Arc<dyn ProductDao>,
}

impl CategoryService {
    // Category Management
    pub async fn create_category(&self, request: CreateCategoryRequest) -> Result<Category, ServiceError>;
    pub async fn get_category_details(&self, id: &str) -> Result<CategoryResponse, ServiceError>;
    pub async fn update_category(&self, id: &str, request: UpdateCategoryRequest) -> Result<Category, ServiceError>;
    pub async fn delete_category(&self, id: &str) -> Result<bool, ServiceError>;
    
    // Hierarchy Navigation
    pub async fn get_category_tree(&self) -> Result<CategoryTreeResponse, ServiceError>;
    pub async fn get_category_path(&self, category_id: &str) -> Result<Vec<Category>, ServiceError>;
    pub async fn get_subcategories(&self, parent_id: &str) -> Result<Vec<Category>, ServiceError>;
    
    // Administrative
    pub async fn rebuild_hierarchy(&self) -> Result<bool, ServiceError>;
    pub async fn move_category(&self, category_id: &str, new_parent_id: Option<&str>) -> Result<bool, ServiceError>;
    
    // Import/Export Operations
    pub async fn export_categories(&self, batch_size: Option<i64>) -> Result<Vec<Category>, ServiceError>;
    pub async fn import_categories(&self, categories: Vec<Category>, dry_run: bool) -> Result<ImportResult, ServiceError>;
}

#[derive(Debug)]
pub struct ImportResult {
    pub successful_imports: usize,
    pub failed_imports: usize,
    pub total_processed: usize,
    pub errors: Vec<String>,
}
```

---

## 5. Protocol Buffers

### 5.1 Category Messages (`proto/category.proto`)

**New proto file to create:**
```protobuf
syntax = "proto3";

package catalog;

import "status.proto";
import "google/protobuf/timestamp.proto";

// Category CRUD Messages
message CreateCategoryRequest {
    string name = 1;
    string slug = 2;
    string short_description = 3;
    optional string full_description = 4;
    optional string parent_id = 5;
    int32 display_order = 6;
    CategorySeo seo = 7;
}

message UpdateCategoryRequest {
    string id = 1;
    optional string name = 2;
    optional string slug = 3;
    optional string short_description = 4;
    optional string full_description = 5;
    optional int32 display_order = 6;
    optional CategorySeo seo = 7;
    optional bool is_active = 8;
}

message CategoryResponse {
    string id = 1;
    string slug = 2;
    string name = 3;
    string short_description = 4;
    optional string full_description = 5;
    string path = 6;
    repeated string ancestors = 7;
    optional string parent_id = 8;
    int32 level = 9;
    int32 children_count = 10;
    int32 product_count = 11;
    bool is_active = 12;
    int32 display_order = 13;
    CategorySeo seo = 14;
    google.protobuf.Timestamp created_at = 15;
    google.protobuf.Timestamp updated_at = 16;
}

message CategorySeo {
    optional string meta_title = 1;
    optional string meta_description = 2;
    repeated string keywords = 3;
}

// Tree Operations
message CategoryTreeRequest {
    optional int32 max_depth = 1;
    optional bool include_inactive = 2;
}

message CategoryTreeResponse {
    repeated CategoryTreeNode tree = 1;
    Status status = 2;
}

message CategoryTreeNode {
    string id = 1;
    string name = 2;
    string slug = 3;
    int32 level = 4;
    int32 product_count = 5;
    repeated CategoryTreeNode children = 6;
}

// Hierarchy Operations
message MoveCategoryRequest {
    string category_id = 1;
    optional string new_parent_id = 2;
}

message GetCategoryPathRequest {
    string category_id = 1;
}

message CategoryPathResponse {
    repeated CategoryResponse path = 1;
    Status status = 2;
}

// Import/Export Operations
message CategoryExportRequest {
    optional int32 batch_size = 1;
    optional int32 offset = 2;
}

message CategoryExportResponse {
    repeated CategoryResponse categories = 1;
    Status status = 2;
}

message CategoryImportRequest {
    repeated CreateCategoryRequest categories = 1;
    bool dry_run = 2;
}

message CategoryImportResponse {
    int32 successful_imports = 1;
    int32 failed_imports = 2;
    int32 total_processed = 3;
    repeated string errors = 4;
    Status status = 5;
}
```

### 5.2 Update Main Catalog Proto

**Add to `proto/catalog.proto`:**
```protobuf
// Import category messages
import "category.proto";

// Add category service methods
service CatalogService {
    // Existing product methods...
    
    // Category methods
    rpc CreateCategory(CreateCategoryRequest) returns (CategoryResponse);
    rpc GetCategory(GetCategoryRequest) returns (CategoryResponse);
    rpc UpdateCategory(UpdateCategoryRequest) returns (CategoryResponse);
    rpc DeleteCategory(DeleteCategoryRequest) returns (Status);
    rpc GetCategoryTree(CategoryTreeRequest) returns (CategoryTreeResponse);
    rpc GetCategoryPath(GetCategoryPathRequest) returns (CategoryPathResponse);
    rpc MoveCategory(MoveCategoryRequest) returns (Status);
    rpc ExportCategories(CategoryExportRequest) returns (CategoryExportResponse);
    rpc ImportCategories(CategoryImportRequest) returns (CategoryImportResponse);
}
```

---

## 6. NATS Message Handlers

### 6.1 Category Handlers (`src/handlers/category_handlers.rs`)

**New file to create:**
```rust
pub async fn handle_create_category(
    msg: &Message,
    category_service: Arc<CategoryService>,
) -> Result<Vec<u8>, Box<dyn Error + Send + Sync>> {
    // Deserialize request, call service, serialize response
}

pub async fn handle_get_category_tree(
    msg: &Message,
    category_service: Arc<CategoryService>,
) -> Result<Vec<u8>, Box<dyn Error + Send + Sync>> {
    // Handle tree retrieval with caching
}

pub async fn handle_move_category(
    msg: &Message,
    category_service: Arc<CategoryService>,
) -> Result<Vec<u8>, Box<dyn Error + Send + Sync>> {
    // Handle category moves with path updates
}

pub async fn handle_export_categories(
    msg: &Message,
    category_service: Arc<CategoryService>,
) -> Result<Vec<u8>, Box<dyn Error + Send + Sync>> {
    // Handle category export with pagination (similar to product export)
}

pub async fn handle_import_categories(
    msg: &Message,
    category_service: Arc<CategoryService>,
) -> Result<Vec<u8>, Box<dyn Error + Send + Sync>> {
    // Handle bulk category import with validation and error reporting
}

// Additional handlers for all category operations
```

### 6.2 Update Main Handlers

**Add to `src/handlers/mod.rs`:**
```rust
pub mod category_handlers;
use category_handlers::*;

// Register new handlers in setup function
```

---

## 7. CLI Client Extensions

### 7.1 Category Commands (`src/catalog-client/main.rs`)

**Add new subcommands:**
```rust
#[derive(Parser)]
enum CategoryCommands {
    Create {
        #[arg(long)]
        name: String,
        #[arg(long)]
        slug: String,
        #[arg(long)]
        description: String,
        #[arg(long)]
        parent: Option<String>,
    },
    List {
        #[arg(long)]
        parent: Option<String>,
        #[arg(long)]
        depth: Option<i32>,
    },
    Tree,
    Move {
        #[arg(long)]
        category: String,
        #[arg(long)]
        new_parent: Option<String>,
    },
    Delete {
        #[arg(long)]
        id: String,
    },
    Import {
        #[arg(short, long)]
        file: PathBuf,
        #[arg(short, long, default_value = "false")]
        dry_run: bool,
    },
    Export {
        #[arg(short, long)]
        file: PathBuf,
        #[arg(short, long, default_value = "50")]
        batch_size: i32,
    },
}
```

**Import/Export Implementation Notes:**
- Export: Similar to product export with pagination to handle large category hierarchies
- Import: Support both single category and array of categories from JSON files
- Include validation for hierarchy consistency during import
- Provide detailed progress reporting with success/failure counts
- Support dry-run mode for import validation

---

## 8. Database Indexes

### 8.1 Required Indexes

**MongoDB index creation script:**
```javascript
// Primary category collection indexes
db.categories.createIndex({ "slug": 1 }, { unique: true });
db.categories.createIndex({ "path": 1 });
db.categories.createIndex({ "parent_id": 1 });
db.categories.createIndex({ "ancestors": 1 });
db.categories.createIndex({ "level": 1 });
db.categories.createIndex({ "is_active": 1, "display_order": 1 });
db.categories.createIndex({ "parent_id": 1, "is_active": 1, "display_order": 1 });

// Text search index
db.categories.createIndex({ 
    "name": "text", 
    "short_description": "text", 
    "seo.keywords": "text" 
});

// Cache collection index
db.category_tree_cache.createIndex({ "version": 1 });
```

---

## 9. Implementation Phases

### Phase 1: Foundation (Week 1)
- [ ] Create Category and related model structs
- [ ] Implement basic CategoryDao trait and implementation
- [ ] Add MongoDB collections and indexes
- [ ] Create basic category proto messages

### Phase 2: Core Operations (Week 2)
- [ ] Implement CRUD operations for categories
- [ ] Add path generation and ancestor management
- [ ] Create hierarchy navigation methods
- [ ] Implement tree cache building logic

### Phase 3: Service Layer (Week 3)
- [ ] Create CategoryService with business logic
- [ ] Add validation and error handling
- [ ] Implement category move operations
- [ ] Add product count aggregation
- [ ] Implement UUID v4 generation for category IDs
- [ ] Add export functionality with pagination
- [ ] Add import functionality with validation and error reporting

### Phase 4: NATS Integration (Week 4)
- [ ] Create NATS message handlers
- [ ] Add protobuf serialization/deserialization
- [ ] Integrate with existing catalog service
- [ ] Add comprehensive error handling
- [ ] Implement export/import message handlers

### Phase 5: Client & Testing (Week 5)
- [ ] Extend CLI client with category commands
- [ ] Add category import/export CLI commands (following product patterns)
- [ ] Create comprehensive unit tests
- [ ] Add integration tests with MongoDB
- [ ] Performance testing with large hierarchies
- [ ] Test import/export with various file formats and sizes

### Phase 6: Optimization (Week 6)
- [ ] Implement tree cache refresh strategies
- [ ] Add background product count updates
- [ ] Optimize query performance
- [ ] Add monitoring and metrics

---

## 10. Testing Strategy

### 10.1 Unit Tests
- Category model validation
- Path generation algorithms
- Tree building logic
- DAO method testing with mock MongoDB

### 10.2 Integration Tests
- Full hierarchy operations with real MongoDB
- Cache invalidation scenarios
- Concurrent category modifications
- Large dataset performance tests

### 10.3 Performance Tests
- Tree retrieval benchmarks
- Category move operation timing
- Cache rebuild performance
- Memory usage with large trees

---

## 11. Monitoring & Maintenance

### 11.1 Metrics to Track
- Tree cache hit rates
- Category operation response times
- Product count accuracy
- Cache rebuild frequency

### 11.2 Maintenance Tasks
- Periodic product count validation
- Tree cache optimization
- Index performance monitoring
- Hierarchy consistency checks

---

## 12. Migration Strategy

### 12.1 Data Migration
- Script to convert existing product categories
- Bulk category creation from existing data
- UUID v4 generation for existing categories
- Path and ancestor calculation
- Initial tree cache population

### 12.2 Backward Compatibility
- Maintain existing product category references
- Gradual migration of category associations
- Update product references to use new UUID-based category IDs
- Deprecation timeline for old category fields

### 12.3 Import/Export Migration Tools
- Export existing category data in new format
- Import validation tools for data consistency
- Batch migration utilities for large category sets
- Hierarchy validation and repair tools

---

This implementation plan provides a comprehensive roadmap for adding sophisticated category hierarchy management to the catalog module while maintaining the existing architecture patterns and ensuring scalability for e-commerce use cases.
