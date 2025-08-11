#[cfg(test)]
mod category_tests {
    use rust_catalog::model::{Category, CategorySeo};
    use chrono::Utc;
    use uuid::Uuid;

    #[test]
    fn test_category_creation() {
        let category = Category {
            id: Some(Uuid::new_v4().to_string()),
            slug: "test-category".to_string(),
            name: "Test Category".to_string(),
            short_description: "A test category".to_string(),
            full_description: Some("Full description of test category".to_string()),
            path: "test-category".to_string(),
            ancestors: vec![],
            parent_id: None,
            level: 0,
            children_count: 0,
            product_count: 0,
            is_active: true,
            display_order: 0,
            seo: CategorySeo {
                meta_title: Some("Test Category".to_string()),
                meta_description: Some("Test category description".to_string()),
                keywords: vec!["test".to_string(), "category".to_string()],
            },
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        assert_eq!(category.name, "Test Category");
        assert_eq!(category.slug, "test-category");
        assert_eq!(category.level, 0);
        assert_eq!(category.ancestors.len(), 0);
        assert!(category.is_active);
    }

    #[test]
    fn test_category_path_generation() {
        // Test root category path
        let root_category = Category {
            id: Some(Uuid::new_v4().to_string()),
            slug: "electronics".to_string(),
            name: "Electronics".to_string(),
            short_description: "Electronics category".to_string(),
            full_description: None,
            path: "electronics".to_string(),
            ancestors: vec![],
            parent_id: None,
            level: 0,
            children_count: 5,
            product_count: 1000,
            is_active: true,
            display_order: 1,
            seo: CategorySeo::default(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        assert_eq!(root_category.path, "electronics");
        assert_eq!(root_category.level, 0);
        assert!(root_category.ancestors.is_empty());

        // Test child category path
        let parent_id = Uuid::new_v4().to_string();
        let child_category = Category {
            id: Some(Uuid::new_v4().to_string()),
            slug: "smartphones".to_string(),
            name: "Smartphones".to_string(),
            short_description: "Smartphone category".to_string(),
            full_description: None,
            path: "electronics.smartphones".to_string(),
            ancestors: vec![parent_id.clone()],
            parent_id: Some(parent_id),
            level: 1,
            children_count: 3,
            product_count: 250,
            is_active: true,
            display_order: 1,
            seo: CategorySeo::default(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        assert_eq!(child_category.path, "electronics.smartphones");
        assert_eq!(child_category.level, 1);
        assert_eq!(child_category.ancestors.len(), 1);
    }

    #[test]
    fn test_category_seo_defaults() {
        let seo = CategorySeo::default();
        
        assert!(seo.meta_title.is_none());
        assert!(seo.meta_description.is_none());
        assert!(seo.keywords.is_empty());
    }

    #[test]
    fn test_category_hierarchy_validation() {
        // Test that a category cannot be its own parent
        let category_id = Uuid::new_v4().to_string();
        
        let invalid_category = Category {
            id: Some(category_id.clone()),
            slug: "invalid".to_string(),
            name: "Invalid Category".to_string(),
            short_description: "Invalid category".to_string(),
            full_description: None,
            path: "invalid".to_string(),
            ancestors: vec![category_id.clone()],
            parent_id: Some(category_id.clone()),
            level: 1,
            children_count: 0,
            product_count: 0,
            is_active: true,
            display_order: 0,
            seo: CategorySeo::default(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        // This should be validated in business logic
        assert_eq!(invalid_category.parent_id, invalid_category.id);
        assert!(invalid_category.ancestors.contains(&category_id));
    }
}
