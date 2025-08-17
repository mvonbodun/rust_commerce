use std::time::Instant;
use rust_catalog::model::{Category, CategorySeo};
use chrono::Utc;
use uuid::Uuid;

#[cfg(test)]
mod performance_tests {
    use super::*;

    #[test]
    #[ignore] // Run only when explicitly requested with --ignored
    fn test_large_category_hierarchy_creation() {
        let start = Instant::now();
        
        // Create a large hierarchy structure in memory
        let mut categories = Vec::new();
        
        // Create 10 root categories
        for i in 0..10 {
            let root_id = Uuid::new_v4().to_string();
            let root_category = Category {
                id: Some(root_id.clone()),
                slug: format!("root-category-{}", i),
                name: format!("Root Category {}", i),
                short_description: format!("Root category {} description", i),
                full_description: Some(format!("Full description for root category {}", i)),
                path: format!("root-category-{}", i),
                ancestors: vec![],
                parent_id: None,
                level: 0,
                children_count: 100, // Each root has 100 children
                product_count: 1000,
                is_active: true,
                display_order: i as i32,
                seo: CategorySeo::default(),
                created_at: Utc::now(),
                updated_at: Utc::now(),
            };
            categories.push(root_category);
            
            // Create 100 child categories for each root
            for j in 0..100 {
                let child_id = Uuid::new_v4().to_string();
                let child_category = Category {
                    id: Some(child_id.clone()),
                    slug: format!("child-category-{}-{}", i, j),
                    name: format!("Child Category {}-{}", i, j),
                    short_description: format!("Child category {}-{} description", i, j),
                    full_description: None,
                    path: format!("root-category-{}.child-category-{}-{}", i, i, j),
                    ancestors: vec![root_id.clone()],
                    parent_id: Some(root_id.clone()),
                    level: 1,
                    children_count: 10, // Each child has 10 grandchildren
                    product_count: 10,
                    is_active: true,
                    display_order: j as i32,
                    seo: CategorySeo::default(),
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                };
                categories.push(child_category);
                
                // Create 10 grandchild categories for each child
                for k in 0..10 {
                    let grandchild_id = Uuid::new_v4().to_string();
                    let grandchild_category = Category {
                        id: Some(grandchild_id.clone()),
                        slug: format!("grandchild-category-{}-{}-{}", i, j, k),
                        name: format!("Grandchild Category {}-{}-{}", i, j, k),
                        short_description: format!("Grandchild category {}-{}-{} description", i, j, k),
                        full_description: None,
                        path: format!("root-category-{}.child-category-{}-{}.grandchild-category-{}-{}-{}", i, i, j, i, j, k),
                        ancestors: vec![root_id.clone(), child_id.clone()],
                        parent_id: Some(child_id.clone()),
                        level: 2,
                        children_count: 0,
                        product_count: 1,
                        is_active: true,
                        display_order: k as i32,
                        seo: CategorySeo::default(),
                        created_at: Utc::now(),
                        updated_at: Utc::now(),
                    };
                    categories.push(grandchild_category);
                }
            }
        }
        
        let creation_duration = start.elapsed();
        println!("Created {} categories in {:?}", categories.len(), creation_duration);
        
        // Expected: 10 + (10 * 100) + (10 * 100 * 10) = 10 + 1,000 + 10,000 = 11,010 categories
        assert_eq!(categories.len(), 11_010);
        
        // Test hierarchy traversal performance
        let traversal_start = Instant::now();
        
        // Count categories by level
        let mut level_counts = std::collections::HashMap::new();
        for category in &categories {
            *level_counts.entry(category.level).or_insert(0) += 1;
        }
        
        let traversal_duration = traversal_start.elapsed();
        println!("Traversed hierarchy in {:?}", traversal_duration);
        
        assert_eq!(level_counts.get(&0), Some(&10)); // 10 root categories
        assert_eq!(level_counts.get(&1), Some(&1000)); // 1,000 child categories
        assert_eq!(level_counts.get(&2), Some(&10000)); // 10,000 grandchild categories
        
        // Test path searching performance
        let search_start = Instant::now();
        
        let search_path = "root-category-5.child-category-5-50.grandchild-category-5-50-5";
        let found_category = categories.iter().find(|cat| cat.path == search_path);
        
        let search_duration = search_start.elapsed();
        println!("Found category by path in {:?}", search_duration);
        
        assert!(found_category.is_some());
        assert_eq!(found_category.unwrap().level, 2);
        
        // Performance assertions (should complete within reasonable time)
        assert!(creation_duration.as_millis() < 1000, "Category creation took too long: {:?}", creation_duration);
        assert!(traversal_duration.as_millis() < 100, "Hierarchy traversal took too long: {:?}", traversal_duration);
        assert!(search_duration.as_micros() < 1000, "Path search took too long: {:?}", search_duration);
    }

    #[test]
    #[ignore] // Run only when explicitly requested
    fn test_category_path_generation_performance() {
        let start = Instant::now();
        
        // Test generating many paths
        let paths: Vec<String> = (0..10000).map(|i| {
            let level = i % 3;
            match level {
                0 => format!("category-{}", i),
                1 => format!("parent-{}.category-{}", i / 10, i),
                2 => format!("grandparent-{}.parent-{}.category-{}", i / 100, i / 10, i),
                _ => unreachable!()
            }
        }).collect();
        
        let duration = start.elapsed();
        println!("Generated {} paths in {:?}", paths.len(), duration);
        
        assert_eq!(paths.len(), 10000);
        assert!(duration.as_millis() < 100, "Path generation took too long: {:?}", duration);
        
        // Test that paths are correctly formatted
        assert!(paths[0].starts_with("category-"));
        assert!(paths[1000].contains("."));
        assert!(paths[2000].matches('.').count() >= 1);
    }

    #[test]
    #[ignore] // Run only when explicitly requested
    fn test_ancestor_array_performance() {
        let start = Instant::now();
        
        // Simulate building ancestor arrays for a deep hierarchy
        let mut ancestors_arrays = Vec::new();
        
        for depth in 0..1000 {
            let mut ancestors = Vec::new();
            for _i in 0..depth {
                ancestors.push(Uuid::new_v4().to_string());
            }
            ancestors_arrays.push(ancestors);
        }
        
        let duration = start.elapsed();
        println!("Built {} ancestor arrays in {:?}", ancestors_arrays.len(), duration);
        
        assert_eq!(ancestors_arrays.len(), 1000);
        assert_eq!(ancestors_arrays[0].len(), 0); // Root category has no ancestors
        assert_eq!(ancestors_arrays[999].len(), 999); // Deep category has many ancestors
        
        // Performance assertion
        assert!(duration.as_millis() < 100, "Ancestor array building took too long: {:?}", duration);
    }
}
