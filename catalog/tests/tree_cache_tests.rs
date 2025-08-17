#[cfg(test)]
mod tree_cache_tests {
    use std::collections::HashMap;
    use rust_catalog::model::{Category, CategorySeo, CategoryTreeCache, CategoryTreeNode};
    use chrono::Utc;
    use uuid::Uuid;

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

    /// Helper function to build a tree structure from categories
    fn build_category_tree(categories: Vec<Category>) -> HashMap<String, CategoryTreeNode> {
        // Create a map of all categories by ID for quick lookup
        let mut category_map = HashMap::new();
        for category in &categories {
            if let Some(ref id) = category.id {
                category_map.insert(id.clone(), category);
            }
        }
        
        // Recursive function to build a node and its children
        fn build_node(
            category: &Category,
            category_map: &HashMap<String, &Category>,
            processed: &mut std::collections::HashSet<String>
        ) -> CategoryTreeNode {
            let id = category.id.as_ref().unwrap();
            
            // Prevent infinite loops
            if processed.contains(id) {
                return CategoryTreeNode {
                    id: id.clone(),
                    name: category.name.clone(),
                    slug: category.slug.clone(),
                    path: category.path.clone(),
                    level: category.level,
                    product_count: category.product_count,
                    children: HashMap::new(),
                };
            }
            processed.insert(id.clone());
            
            let mut children = HashMap::new();
            
            // Find all children of this category
            for (child_id, child_category) in category_map {
                if child_category.parent_id.as_ref() == Some(id) {
                    let child_node = build_node(child_category, category_map, processed);
                    children.insert(child_id.clone(), child_node);
                }
            }
            
            CategoryTreeNode {
                id: id.clone(),
                name: category.name.clone(),
                slug: category.slug.clone(),
                path: category.path.clone(),
                level: category.level,
                product_count: category.product_count,
                children,
            }
        }
        
        // Build tree starting from root categories (those without parents)
        let mut tree = HashMap::new();
        let mut processed = std::collections::HashSet::new();
        
        for category in &categories {
            if category.parent_id.is_none() {
                if let Some(ref id) = category.id {
                    let root_node = build_node(category, &category_map, &mut processed);
                    tree.insert(id.clone(), root_node);
                }
            }
        }
        
        tree
    }

    #[test]
    fn test_tree_cache_creation() {
        let categories = vec![
            create_test_category("electronics", "Electronics", None, 0, 1),
            create_test_category("clothing", "Clothing", None, 0, 2),
        ];
        
        let tree = build_category_tree(categories);
        
        let cache = CategoryTreeCache {
            id: "test_tree_v1".to_string(),
            version: 1,
            last_updated: Utc::now(),
            tree,
        };
        
        assert_eq!(cache.version, 1);
        assert_eq!(cache.tree.len(), 2); // Two root categories
        assert!(cache.tree.contains_key(&cache.tree.values().find(|n| n.slug == "electronics").unwrap().id));
        assert!(cache.tree.contains_key(&cache.tree.values().find(|n| n.slug == "clothing").unwrap().id));
    }

    #[test]
    fn test_hierarchical_tree_cache() {
        let mut categories = vec![];
        
        // Create Electronics category
        let electronics = create_test_category("electronics", "Electronics", None, 0, 1);
        let electronics_id = electronics.id.clone().unwrap();
        categories.push(electronics);
        
        // Create subcategories
        categories.push(create_test_category(
            "smartphones", 
            "Smartphones", 
            Some(electronics_id.clone()), 
            1, 
            1
        ));
        categories.push(create_test_category(
            "laptops", 
            "Laptops", 
            Some(electronics_id.clone()), 
            1, 
            2
        ));
        
        let tree = build_category_tree(categories);
        
        let cache = CategoryTreeCache {
            id: "hierarchical_tree_v1".to_string(),
            version: 1,
            last_updated: Utc::now(),
            tree,
        };
        
        // Verify structure
        assert_eq!(cache.tree.len(), 1); // One root category
        
        let electronics_node = cache.tree.get(&electronics_id).unwrap();
        assert_eq!(electronics_node.name, "Electronics");
        assert_eq!(electronics_node.level, 0);
        assert_eq!(electronics_node.children.len(), 2); // Two child categories
        
        // Verify children exist
        let has_smartphones = electronics_node.children.values().any(|child| child.slug == "smartphones");
        let has_laptops = electronics_node.children.values().any(|child| child.slug == "laptops");
        assert!(has_smartphones);
        assert!(has_laptops);
    }

    #[test]
    fn test_deep_hierarchy_tree_cache() {
        let mut categories = vec![];
        
        // Level 0: Electronics
        let electronics = create_test_category("electronics", "Electronics", None, 0, 1);
        let electronics_id = electronics.id.clone().unwrap();
        categories.push(electronics);
        
        // Level 1: Mobile Devices
        let mobile = create_test_category(
            "mobile-devices", 
            "Mobile Devices", 
            Some(electronics_id.clone()), 
            1, 
            1
        );
        let mobile_id = mobile.id.clone().unwrap();
        categories.push(mobile);
        
        // Level 2: Smartphones
        let smartphones = create_test_category(
            "smartphones", 
            "Smartphones", 
            Some(mobile_id.clone()), 
            2, 
            1
        );
        let smartphones_id = smartphones.id.clone().unwrap();
        categories.push(smartphones);
        
        // Level 3: iPhone
        categories.push(create_test_category(
            "iphone", 
            "iPhone", 
            Some(smartphones_id.clone()), 
            3, 
            1
        ));
        
        let tree = build_category_tree(categories);
        
        let cache = CategoryTreeCache {
            id: "deep_hierarchy_tree_v1".to_string(),
            version: 1,
            last_updated: Utc::now(),
            tree,
        };
        
        // Verify 4-level deep structure
        assert_eq!(cache.tree.len(), 1); // One root
        
        let electronics_node = cache.tree.get(&electronics_id).unwrap();
        assert_eq!(electronics_node.children.len(), 1);
        
        let mobile_node = electronics_node.children.get(&mobile_id).unwrap();
        assert_eq!(mobile_node.level, 1);
        assert_eq!(mobile_node.children.len(), 1);
        
        let smartphones_node = mobile_node.children.get(&smartphones_id).unwrap();
        assert_eq!(smartphones_node.level, 2);
        assert_eq!(smartphones_node.children.len(), 1);
        
        let iphone_node = smartphones_node.children.values().next().unwrap();
        assert_eq!(iphone_node.level, 3);
        assert_eq!(iphone_node.name, "iPhone");
        assert_eq!(iphone_node.children.len(), 0); // Leaf node
    }

    #[test]
    fn test_multiple_root_categories_tree_cache() {
        let categories = vec![
            create_test_category("electronics", "Electronics", None, 0, 1),
            create_test_category("clothing", "Clothing", None, 0, 2),
            create_test_category("books", "Books", None, 0, 3),
        ];
        
        let tree = build_category_tree(categories);
        
        let cache = CategoryTreeCache {
            id: "multi_root_tree_v1".to_string(),
            version: 1,
            last_updated: Utc::now(),
            tree,
        };
        
        // Verify all root categories exist
        assert_eq!(cache.tree.len(), 3);
        
        let category_names: Vec<String> = cache.tree.values()
            .map(|node| node.name.clone())
            .collect();
        
        assert!(category_names.contains(&"Electronics".to_string()));
        assert!(category_names.contains(&"Clothing".to_string()));
        assert!(category_names.contains(&"Books".to_string()));
        
        // Verify all are level 0 (root level)
        for node in cache.tree.values() {
            assert_eq!(node.level, 0);
            assert_eq!(node.children.len(), 0); // No children in this test
        }
    }

    #[test]
    fn test_tree_cache_versioning() {
        let categories = vec![
            create_test_category("electronics", "Electronics", None, 0, 1),
        ];
        
        let tree = build_category_tree(categories);
        
        let cache_v1 = CategoryTreeCache {
            id: "version_test_v1".to_string(),
            version: 1,
            last_updated: Utc::now(),
            tree: tree.clone(),
        };
        
        let cache_v2 = CategoryTreeCache {
            id: "version_test_v1".to_string(), // Same ID
            version: 2, // Different version
            last_updated: Utc::now(),
            tree,
        };
        
        assert_eq!(cache_v1.id, cache_v2.id);
        assert_ne!(cache_v1.version, cache_v2.version);
        assert!(cache_v2.version > cache_v1.version);
    }

    #[test]
    fn test_tree_cache_performance_structure() {
        // Test with a realistic e-commerce category structure
        let mut categories = vec![];
        
        // Electronics root
        let electronics = create_test_category("electronics", "Electronics", None, 0, 1);
        let electronics_id = electronics.id.clone().unwrap();
        categories.push(electronics);
        
        // Major subcategories
        let computers = create_test_category("computers", "Computers", Some(electronics_id.clone()), 1, 1);
        let computers_id = computers.id.clone().unwrap();
        categories.push(computers);
        
        let phones = create_test_category("phones", "Phones", Some(electronics_id.clone()), 1, 2);
        let phones_id = phones.id.clone().unwrap();
        categories.push(phones);
        
        // Computer subcategories
        categories.push(create_test_category("laptops", "Laptops", Some(computers_id.clone()), 2, 1));
        categories.push(create_test_category("desktops", "Desktops", Some(computers_id.clone()), 2, 2));
        categories.push(create_test_category("tablets", "Tablets", Some(computers_id.clone()), 2, 3));
        
        // Phone subcategories
        categories.push(create_test_category("smartphones", "Smartphones", Some(phones_id.clone()), 2, 1));
        categories.push(create_test_category("feature-phones", "Feature Phones", Some(phones_id.clone()), 2, 2));
        
        let tree = build_category_tree(categories);
        
        let cache = CategoryTreeCache {
            id: "performance_tree_v1".to_string(),
            version: 1,
            last_updated: Utc::now(),
            tree,
        };
        
        // Verify the structure is correct
        assert_eq!(cache.tree.len(), 1); // One root (Electronics)
        
        let electronics_node = cache.tree.get(&electronics_id).unwrap();
        assert_eq!(electronics_node.children.len(), 2); // Computers and Phones
        
        let computers_node = electronics_node.children.get(&computers_id).unwrap();
        assert_eq!(computers_node.children.len(), 3); // Laptops, Desktops, Tablets
        
        let phones_node = electronics_node.children.get(&phones_id).unwrap();
        assert_eq!(phones_node.children.len(), 2); // Smartphones, Feature Phones
        
        // Verify we can efficiently access any category through the tree
        assert!(electronics_node.children.values().any(|child| child.name == "Computers"));
        assert!(electronics_node.children.values().any(|child| child.name == "Phones"));
        
        let computers_children: Vec<&str> = computers_node.children.values()
            .map(|child| child.name.as_str())
            .collect();
        assert!(computers_children.contains(&"Laptops"));
        assert!(computers_children.contains(&"Desktops"));
        assert!(computers_children.contains(&"Tablets"));
    }
}
