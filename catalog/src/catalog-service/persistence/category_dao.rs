use async_trait::async_trait;
use mongodb::{Collection, bson::doc};
use crate::model::{Category, CategoryTreeCache};
use std::error::Error;
use futures::TryStreamExt;
use uuid::Uuid;
use chrono::Utc;

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

pub struct CategoryDaoImpl {
    collection: Collection<Category>,
    cache_collection: Collection<CategoryTreeCache>,
}

impl CategoryDaoImpl {
    pub fn new(collection: Collection<Category>, cache_collection: Collection<CategoryTreeCache>) -> Self {
        Self { 
            collection,
            cache_collection,
        }
    }

    /// Helper method to calculate hierarchy data for a category
    async fn calculate_hierarchy_data(&self, mut category: Category) -> Result<Category, Box<dyn Error + Send + Sync>> {
        if let Some(parent_id) = &category.parent_id {
            // Get parent category
            if let Some(parent) = self.get_category(parent_id).await? {
                // Build ancestors array
                let mut ancestors = parent.ancestors.clone();
                ancestors.push(parent_id.clone());
                category.ancestors = ancestors;
                
                // Calculate level
                category.level = category.calculate_level();
                
                // Get all ancestors to build path
                let ancestor_categories = self.get_ancestors_by_ids(&category.ancestors).await?;
                category.path = category.generate_path(&ancestor_categories);
            } else {
                return Err(format!("Parent category with ID {} not found", parent_id).into());
            }
        } else {
            // Root category
            category.ancestors = Vec::new();
            category.level = 0;
            category.path = category.slug.clone();
        }
        
        Ok(category)
    }

    /// Helper method to get ancestor categories by their IDs
    async fn get_ancestors_by_ids(&self, ancestor_ids: &[String]) -> Result<Vec<Category>, Box<dyn Error + Send + Sync>> {
        if ancestor_ids.is_empty() {
            return Ok(Vec::new());
        }

        let filter = doc! { "_id": { "$in": ancestor_ids } };
        let cursor = self.collection.find(filter).await?;
        let ancestors: Vec<Category> = cursor.try_collect().await?;
        
        // Sort ancestors by level to ensure correct order
        let mut sorted_ancestors = ancestors;
        sorted_ancestors.sort_by_key(|cat| cat.level);
        
        Ok(sorted_ancestors)
    }

    /// Helper method to update children count for a parent category
    async fn update_children_count(&self, parent_id: &str) -> Result<(), Box<dyn Error + Send + Sync>> {
        let children_count = self.collection
            .count_documents(doc! { "parent_id": parent_id })
            .await? as i32;

        self.collection
            .update_one(
                doc! { "_id": parent_id },
                doc! { "$set": { "children_count": children_count } },
            )
            .await?;

        Ok(())
    }
}

#[async_trait]
impl CategoryDao for CategoryDaoImpl {
    async fn create_category(&self, mut category: Category) -> Result<Category, Box<dyn Error + Send + Sync>> {
        // Generate UUID if not provided
        if category.id.is_none() {
            category.id = Some(Uuid::new_v4().to_string());
        }

        // Calculate hierarchy data
        category = self.calculate_hierarchy_data(category).await?;
        
        // Set timestamps
        let now = Utc::now();
        category.created_at = now;
        category.updated_at = now;

        // Insert the category
        self.collection.insert_one(&category).await?;

        // Update parent's children count if this is not a root category
        if let Some(parent_id) = &category.parent_id {
            self.update_children_count(parent_id).await?;
        }

        // Invalidate tree cache
        self.invalidate_tree_cache().await?;

        Ok(category)
    }

    async fn get_category(&self, id: &str) -> Result<Option<Category>, Box<dyn Error + Send + Sync>> {
        let category = self.collection.find_one(doc! { "_id": id }).await?;
        Ok(category)
    }

    async fn get_category_by_slug(&self, slug: &str) -> Result<Option<Category>, Box<dyn Error + Send + Sync>> {
        let category = self.collection.find_one(doc! { "slug": slug }).await?;
        Ok(category)
    }

    async fn update_category(&self, id: &str, mut category: Category) -> Result<Option<Category>, Box<dyn Error + Send + Sync>> {
        // Get the existing category to check if parent changed
        let existing = self.get_category(id).await?;
        if existing.is_none() {
            return Ok(None);
        }
        let existing_category = existing.unwrap();

        // Check if parent changed - if so, recalculate hierarchy
        if existing_category.parent_id != category.parent_id {
            category = self.calculate_hierarchy_data(category).await?;
        }

        // Preserve original timestamps and update modified timestamp
        category.created_at = existing_category.created_at;
        category.updated_at = Utc::now();

        // Update the category
        let result = self.collection.replace_one(
            doc! { "_id": id },
            &category,
        ).await?;

        if result.modified_count > 0 {
            // Update children counts if parent changed
            if existing_category.parent_id != category.parent_id {
                // Update old parent's count
                if let Some(old_parent_id) = &existing_category.parent_id {
                    self.update_children_count(old_parent_id).await?;
                }
                // Update new parent's count
                if let Some(new_parent_id) = &category.parent_id {
                    self.update_children_count(new_parent_id).await?;
                }
            }

            // Invalidate tree cache
            self.invalidate_tree_cache().await?;

            Ok(Some(category))
        } else {
            Ok(None)
        }
    }

    async fn delete_category(&self, id: &str) -> Result<bool, Box<dyn Error + Send + Sync>> {
        // Check if category has children
        let children_count = self.collection
            .count_documents(doc! { "parent_id": id })
            .await?;

        if children_count > 0 {
            return Err(format!("Cannot delete category with {} children. Move or delete children first.", children_count).into());
        }

        // Get category to update parent's children count
        let category = self.get_category(id).await?;
        
        let result = self.collection.delete_one(doc! { "_id": id }).await?;
        
        if result.deleted_count > 0 {
            // Update parent's children count if this wasn't a root category
            if let Some(category) = category {
                if let Some(parent_id) = &category.parent_id {
                    self.update_children_count(parent_id).await?;
                }
            }

            // Invalidate tree cache
            self.invalidate_tree_cache().await?;

            Ok(true)
        } else {
            Ok(false)
        }
    }

    async fn get_children(&self, parent_id: &str) -> Result<Vec<Category>, Box<dyn Error + Send + Sync>> {
        let cursor = self.collection
            .find(doc! { "parent_id": parent_id })
            .sort(doc! { "display_order": 1, "name": 1 })
            .await?;

        let children: Vec<Category> = cursor.try_collect().await?;
        Ok(children)
    }

    async fn get_descendants(&self, ancestor_id: &str) -> Result<Vec<Category>, Box<dyn Error + Send + Sync>> {
        let cursor = self.collection
            .find(doc! { "ancestors": ancestor_id })
            .sort(doc! { "level": 1, "display_order": 1, "name": 1 })
            .await?;

        let descendants: Vec<Category> = cursor.try_collect().await?;
        Ok(descendants)
    }

    async fn get_ancestors(&self, category_id: &str) -> Result<Vec<Category>, Box<dyn Error + Send + Sync>> {
        let category = self.get_category(category_id).await?;
        if let Some(cat) = category {
            self.get_ancestors_by_ids(&cat.ancestors).await
        } else {
            Ok(Vec::new())
        }
    }

    async fn get_breadcrumbs(&self, category_id: &str) -> Result<Vec<Category>, Box<dyn Error + Send + Sync>> {
        let mut breadcrumbs = self.get_ancestors(category_id).await?;
        
        // Add the current category
        if let Some(current) = self.get_category(category_id).await? {
            breadcrumbs.push(current);
        }
        
        Ok(breadcrumbs)
    }

    async fn move_category(&self, category_id: &str, new_parent_id: Option<&str>) -> Result<bool, Box<dyn Error + Send + Sync>> {
        // Get the category to move
        let mut category = match self.get_category(category_id).await? {
            Some(cat) => cat,
            None => return Ok(false),
        };

        let old_parent_id = category.parent_id.clone();
        
        // Update parent
        category.parent_id = new_parent_id.map(|s| s.to_string());
        
        // Recalculate hierarchy
        category = self.calculate_hierarchy_data(category).await?;
        
        // Update the category
        self.update_category(category_id, category).await?;

        // Update children counts
        if let Some(old_parent) = &old_parent_id {
            self.update_children_count(old_parent).await?;
        }
        if let Some(new_parent) = new_parent_id {
            self.update_children_count(new_parent).await?;
        }

        // TODO: Update all descendants' paths and ancestor arrays
        // This is a complex operation that would require recursive updates
        
        Ok(true)
    }

    async fn get_full_tree(&self) -> Result<Option<CategoryTreeCache>, Box<dyn Error + Send + Sync>> {
        let tree_cache = self.cache_collection
            .find_one(doc! {})
            .sort(doc! { "version": -1 })
            .await?;
        
        Ok(tree_cache)
    }

    async fn rebuild_tree_cache(&self) -> Result<CategoryTreeCache, Box<dyn Error + Send + Sync>> {
        // Get all categories
        let cursor = self.collection
            .find(doc! { "is_active": true })
            .sort(doc! { "level": 1, "display_order": 1 })
            .await?;
        
        let categories: Vec<Category> = cursor.try_collect().await?;
        
        // Build tree structure
        // This is a simplified implementation - in practice, you'd want more sophisticated tree building
        let tree = std::collections::HashMap::new(); // TODO: Implement tree building logic
        
        let cache = CategoryTreeCache {
            id: "category_tree_v1".to_string(),
            version: 1,
            last_updated: Utc::now(),
            tree,
        };

        // Save to cache collection
        self.cache_collection
            .replace_one(
                doc! { "_id": &cache.id },
                &cache,
            )
            .await?;

        Ok(cache)
    }

    async fn invalidate_tree_cache(&self) -> Result<bool, Box<dyn Error + Send + Sync>> {
        let result = self.cache_collection
            .delete_many(doc! {})
            .await?;
        
        Ok(result.deleted_count > 0)
    }

    async fn update_product_counts(&self) -> Result<bool, Box<dyn Error + Send + Sync>> {
        // TODO: Implement product count aggregation
        // This would require joining with the products collection
        Ok(true)
    }

    async fn reorder_children(&self, parent_id: &str, ordered_ids: Vec<String>) -> Result<bool, Box<dyn Error + Send + Sync>> {
        for (index, category_id) in ordered_ids.iter().enumerate() {
            self.collection
                .update_one(
                    doc! { "_id": category_id, "parent_id": parent_id },
                    doc! { "$set": { "display_order": index as i32 + 1 } },
                )
                .await?;
        }

        // Invalidate tree cache
        self.invalidate_tree_cache().await?;

        Ok(true)
    }

    async fn export_all_categories(&self, batch_size: Option<i64>) -> Result<Vec<Category>, Box<dyn Error + Send + Sync>> {
        let batch_size = batch_size.unwrap_or(50);
        
        let cursor = self.collection
            .find(doc! {})
            .sort(doc! { "level": 1, "display_order": 1 })
            .limit(batch_size)
            .await?;
        
        let categories: Vec<Category> = cursor.try_collect().await?;
        
        Ok(categories)
    }

    async fn export_categories_batch(&self, batch_size: Option<i64>, offset: Option<u64>) -> Result<Vec<Category>, Box<dyn Error + Send + Sync>> {
        let batch_size = batch_size.unwrap_or(50);
        let offset = offset.unwrap_or(0);
        
        let cursor = self.collection
            .find(doc! {})
            .sort(doc! { "level": 1, "display_order": 1 })
            .limit(batch_size)
            .skip(offset)
            .await?;
        
        let categories: Vec<Category> = cursor.try_collect().await?;
        
        Ok(categories)
    }
}

#[cfg(test)]
mod tests {
    // Note: These tests would require a MongoDB test instance
    // For now, they serve as documentation of expected behavior

    #[tokio::test]
    #[ignore] // Ignore until we have test MongoDB setup
    async fn test_create_root_category() {
        // This test would verify creating a root category
        // and ensuring proper hierarchy calculation
    }

    #[tokio::test] 
    #[ignore]
    async fn test_create_child_category() {
        // This test would verify creating a child category
        // and ensuring proper path and ancestor calculation
    }

    #[tokio::test]
    #[ignore]
    async fn test_move_category() {
        // This test would verify moving a category
        // and updating all hierarchy-related fields
    }

    #[tokio::test]
    #[ignore]
    async fn test_delete_category_with_children() {
        // This test would verify that deleting a category
        // with children properly fails
    }
}
