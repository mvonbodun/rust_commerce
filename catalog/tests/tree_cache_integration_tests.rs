#[cfg(test)]
mod tree_cache_integration_tests {
    use std::collections::HashMap;
    use rust_catalog::model::{Category, CategorySeo, CategoryTreeCache};
    use rust_catalog::persistence::category_dao::{CategoryDao, CategoryDaoImpl};
    use chrono::Utc;
    use uuid::Uuid;
    use mongodb::{Client, Collection};

    /// Helper function to create a test category
    fn create_test_category(
        slug: &str,
        name: &str,
        parent_id: Option<String>,
        level: i32,
        display_order: i32,
    ) -> Category {
        Category {
            id: Some(Uuid::new_v4().to_string()),
            slug: slug.to_string(),
            name: name.to_string(),
            short_description: format!("{} description", name),
            full_description: Some(format!("Full description for {}", name)),
            path: slug.to_string(),
            ancestors: vec![],
            parent_id,
            level,
            children_count: 0,
            product_count: 0,
            is_active: true,
            display_order,
            seo: CategorySeo::default(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[tokio::test]
    #[ignore] // Requires MongoDB connection
    async fn test_build_tree_cache_integration() {
        // Connect to MongoDB test database
        let client = Client::with_uri_str("mongodb://localhost:27017")
            .await
            .expect("Failed to connect to MongoDB");
        
        let db = client.database("test_catalog");
        let categories_collection: Collection<Category> = db.collection("categories");
        let cache_collection: Collection<CategoryTreeCache> = db.collection("category_tree_cache");
        
        // Clean up any existing data
        categories_collection.drop().await.ok();
        cache_collection.drop().await.ok();
        
        let dao = CategoryDaoImpl::new(categories_collection, cache_collection);
        
        // Create test categories
        let electronics = create_test_category("electronics", "Electronics", None, 0, 1);
        let electronics_id = electronics.id.clone().unwrap();
        let created_electronics = dao.create_category(electronics).await.unwrap();
        
        let phones = create_test_category(
            "phones", 
            "Phones", 
            Some(electronics_id.clone()), 
            1, 
            1
        );
        let phones_id = phones.id.clone().unwrap();
        let created_phones = dao.create_category(phones).await.unwrap();
        
        let smartphones = create_test_category(
            "smartphones", 
            "Smartphones", 
            Some(phones_id.clone()), 
            2, 
            1
        );
        let created_smartphones = dao.create_category(smartphones).await.unwrap();
        
        // Build tree cache
        let tree_cache = dao.rebuild_tree_cache().await.unwrap();
        
        // Verify tree structure
        assert_eq!(tree_cache.tree.len(), 1); // One root category
        assert_eq!(tree_cache.version, 1);
        assert_eq!(tree_cache.id, "category_tree_v1");
        
        // Verify Electronics is the root
        let electronics_node = tree_cache.tree.get(&electronics_id).unwrap();
        assert_eq!(electronics_node.name, "Electronics");
        assert_eq!(electronics_node.level, 0);
        assert_eq!(electronics_node.children.len(), 1);
        
        // Verify Phones is a child of Electronics
        let phones_node = electronics_node.children.get(&phones_id).unwrap();
        assert_eq!(phones_node.name, "Phones");
        assert_eq!(phones_node.level, 1);
        assert_eq!(phones_node.children.len(), 1);
        
        // Verify Smartphones is a child of Phones
        let smartphones_node = phones_node.children.values().next().unwrap();
        assert_eq!(smartphones_node.name, "Smartphones");
        assert_eq!(smartphones_node.level, 2);
        assert_eq!(smartphones_node.children.len(), 0);
        
        println!("✅ Tree cache integration test passed!");
        println!("   🌳 Root categories: {}", tree_cache.tree.len());
        println!("   📦 Electronics children: {}", electronics_node.children.len());
        println!("   📱 Phones children: {}", phones_node.children.len());
    }

    #[tokio::test]
    #[ignore] // Requires MongoDB connection
    async fn test_get_full_tree_cache() {
        // Connect to MongoDB test database
        let client = Client::with_uri_str("mongodb://localhost:27017")
            .await
            .expect("Failed to connect to MongoDB");
        
        let db = client.database("test_catalog");
        let categories_collection: Collection<Category> = db.collection("categories");
        let cache_collection: Collection<CategoryTreeCache> = db.collection("category_tree_cache");
        
        // Clean up any existing data
        categories_collection.drop().await.ok();
        cache_collection.drop().await.ok();
        
        let dao = CategoryDaoImpl::new(categories_collection, cache_collection);
        
        // Create and populate test data
        let electronics = create_test_category("electronics", "Electronics", None, 0, 1);
        let clothing = create_test_category("clothing", "Clothing", None, 0, 2);
        
        dao.create_category(electronics).await.unwrap();
        dao.create_category(clothing).await.unwrap();
        
        // Build initial cache
        dao.rebuild_tree_cache().await.unwrap();
        
        // Test retrieving the full tree
        let retrieved_cache = dao.get_full_tree().await.unwrap();
        
        assert!(retrieved_cache.is_some());
        let cache = retrieved_cache.unwrap();
        
        assert_eq!(cache.tree.len(), 2); // Two root categories
        assert_eq!(cache.version, 1);
        
        // Verify both root categories exist
        let has_electronics = cache.tree.values().any(|node| node.name == "Electronics");
        let has_clothing = cache.tree.values().any(|node| node.name == "Clothing");
        
        assert!(has_electronics);
        assert!(has_clothing);
        
        println!("✅ Get full tree cache test passed!");
        println!("   🌳 Retrieved {} root categories", cache.tree.len());
    }

    #[tokio::test]
    #[ignore] // Requires MongoDB connection
    async fn test_tree_cache_invalidation() {
        // Connect to MongoDB test database
        let client = Client::with_uri_str("mongodb://localhost:27017")
        .await
        .expect("Failed to connect to MongoDB");
        
        let db = client.database("test_catalog");
        let categories_collection: Collection<Category> = db.collection("categories");
        let cache_collection: Collection<CategoryTreeCache> = db.collection("category_tree_cache");
        
        // Clean up any existing data
        categories_collection.drop().await.ok();
        cache_collection.drop().await.ok();
        
        let dao = CategoryDaoImpl::new(categories_collection, cache_collection);
        
        // Create test data and cache
        let electronics = create_test_category("electronics", "Electronics", None, 0, 1);
        dao.create_category(electronics).await.unwrap();
        dao.rebuild_tree_cache().await.unwrap();
        
        // Verify cache exists
        let cache_before = dao.get_full_tree().await.unwrap();
        assert!(cache_before.is_some());
        
        // Invalidate cache
        let invalidated = dao.invalidate_tree_cache().await.unwrap();
        assert!(invalidated);
        
        // Verify cache is gone
        let cache_after = dao.get_full_tree().await.unwrap();
        assert!(cache_after.is_none());
        
        println!("✅ Tree cache invalidation test passed!");
    }

    #[tokio::test]
    #[ignore] // Requires MongoDB connection  
    async fn test_tree_cache_rebuild_performance() {
        // Connect to MongoDB test database
        let client = Client::with_uri_str("mongodb://localhost:27017")
            .await
            .expect("Failed to connect to MongoDB");
        
        let db = client.database("test_catalog");
        let categories_collection: Collection<Category> = db.collection("categories");
        let cache_collection: Collection<CategoryTreeCache> = db.collection("category_tree_cache");
        
        // Clean up any existing data
        categories_collection.drop().await.ok();
        cache_collection.drop().await.ok();
        
        let dao = CategoryDaoImpl::new(categories_collection, cache_collection);
        
        // Create a larger category hierarchy for performance testing
        let mut created_categories = Vec::new();
        
        // Create 10 root categories
        for i in 0..10 {
            let root = create_test_category(
                &format!("root-{}", i),
                &format!("Root Category {}", i),
                None,
                0,
                i
            );
            let created_root = dao.create_category(root).await.unwrap();
            created_categories.push(created_root);
        }
        
        // Create 5 children for each root (50 total)
        for root in &created_categories {
            for j in 0..5 {
                let child = create_test_category(
                    &format!("{}-child-{}", root.slug, j),
                    &format!("{} Child {}", root.name, j),
                    root.id.clone(),
                    1,
                    j
                );
                dao.create_category(child).await.unwrap();
            }
        }
        
        // Measure tree cache build time
        let start_time = std::time::Instant::now();
        let tree_cache = dao.rebuild_tree_cache().await.unwrap();
        let build_time = start_time.elapsed();
        
        // Verify the cache structure
        assert_eq!(tree_cache.tree.len(), 10); // 10 root categories
        
        // Verify each root has 5 children
        for (_, root_node) in &tree_cache.tree {
            assert_eq!(root_node.children.len(), 5);
            assert_eq!(root_node.level, 0);
            
            // Verify children have correct level
            for (_, child_node) in &root_node.children {
                assert_eq!(child_node.level, 1);
                assert_eq!(child_node.children.len(), 0); // No grandchildren
            }
        }
        
        println!("✅ Tree cache performance test passed!");
        println!("   ⏱️  Build time: {:?}", build_time);
        println!("   🌳 Root categories: {}", tree_cache.tree.len());
        println!("   📦 Total categories processed: ~60");
        
        // Performance assertion - should build cache for 60 categories in under 100ms
        assert!(build_time.as_millis() < 100, "Tree cache build took too long: {:?}", build_time);
    }
}
