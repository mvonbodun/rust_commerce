use std::sync::Arc;
use log::debug;
use crate::{
    model::{Category, CategorySeo},
    persistence::category_dao::CategoryDao,
    catalog_messages::{CreateCategoryRequest, CategoryResponse, UpdateCategoryRequest},
};

pub struct CategoryService {
    category_dao: Arc<dyn CategoryDao + Send + Sync>,
}

#[derive(Debug)]
pub struct ImportResult {
    pub successful_imports: usize,
    pub failed_imports: usize,
    pub total_processed: usize,
    pub errors: Vec<String>,
}

impl CategoryService {
    pub fn new(category_dao: Arc<dyn CategoryDao + Send + Sync>) -> Self {
        Self { category_dao }
    }

    /// Create a new category
    pub async fn create_category(&self, request: CreateCategoryRequest) -> Result<CategoryResponse, Box<dyn std::error::Error + Send + Sync>> {
        // Validate input
        if request.name.trim().is_empty() {
            return Err("Category name cannot be empty".into());
        }
        
        if request.slug.trim().is_empty() {
            return Err("Category slug cannot be empty".into());
        }

        // Check if slug already exists
        if let Some(_) = self.category_dao.get_category_by_slug(&request.slug).await? {
            return Err(format!("Category with slug '{}' already exists", request.slug).into());
        }

        // Create category model
        let mut category = Category::new(
            request.slug.clone(),
            request.name.clone(),
            request.short_description.clone(),
            request.parent_id.clone(),
            request.display_order,
        );

        // Set optional fields
        category.full_description = request.full_description.clone();
        
        // Set SEO data
        if let Some(seo) = request.seo {
            category.seo = CategorySeo {
                meta_title: seo.meta_title,
                meta_description: seo.meta_description,
                keywords: seo.keywords,
            };
        } else {
            category.seo = CategorySeo::default_for_category(&request.name, &request.short_description);
        }

        // Create category
        let created_category = self.category_dao.create_category(category).await?;

        // Convert to response
        Ok(self.category_to_response(created_category))
    }

    /// Get category by ID
    pub async fn get_category(&self, id: &str) -> Result<Option<CategoryResponse>, Box<dyn std::error::Error + Send + Sync>> {
        match self.category_dao.get_category(id).await? {
            Some(category) => Ok(Some(self.category_to_response(category))),
            None => Ok(None),
        }
    }

    /// Get category by slug
    pub async fn get_category_by_slug(&self, slug: &str) -> Result<Option<CategoryResponse>, Box<dyn std::error::Error + Send + Sync>> {
        match self.category_dao.get_category_by_slug(slug).await? {
            Some(category) => Ok(Some(self.category_to_response(category))),
            None => Ok(None),
        }
    }

    /// Export categories with pagination
    pub async fn export_categories(&self, batch_size: Option<i64>, offset: Option<u64>) -> Result<Vec<CategoryResponse>, Box<dyn std::error::Error + Send + Sync>> {
        let categories = if let Some(offset) = offset {
            self.category_dao.export_categories_batch(batch_size, Some(offset)).await?
        } else {
            self.category_dao.export_all_categories(batch_size).await?
        };

        Ok(categories.into_iter().map(|cat| self.category_to_response(cat)).collect())
    }

    /// Update an existing category
    pub async fn update_category(&self, request: UpdateCategoryRequest) -> Result<CategoryResponse, Box<dyn std::error::Error + Send + Sync>> {
        debug!("Updating category with ID: {}", request.id);

        // Validate required fields
        if request.id.is_empty() {
            return Err("Category ID is required".into());
        }

        // Check if category exists
        let existing_category = self.category_dao.get_category(&request.id).await?
            .ok_or("Category not found")?;

        // Create updated category from request
        let mut updated_category = existing_category.clone();
        
        if let Some(name) = request.name {
            if !name.is_empty() {
                updated_category.name = name;
            }
        }
        
        if let Some(short_description) = request.short_description {
            if !short_description.is_empty() {
                updated_category.short_description = short_description;
            }
        }
        
        if let Some(full_desc) = request.full_description {
            updated_category.full_description = Some(full_desc);
        }
        
        // Handle slug update (regenerate if name changed or explicit slug provided)
        if let Some(new_slug) = request.slug {
            if !new_slug.is_empty() {
                // Check if slug is already taken by another category
                if let Some(existing_with_slug) = self.category_dao.get_category_by_slug(&new_slug).await? {
                    if existing_with_slug.id != Some(request.id.clone()) {
                        return Err("Slug already exists".into());
                    }
                }
                updated_category.slug = new_slug;
            }
        }
        
        if let Some(is_active) = request.is_active {
            updated_category.is_active = is_active;
        }
        
        if let Some(display_order) = request.display_order {
            updated_category.display_order = display_order;
        }
        
        // Handle SEO update
        if let Some(seo_request) = request.seo {
            updated_category.seo.meta_title = seo_request.meta_title;
            updated_category.seo.meta_description = seo_request.meta_description;
            updated_category.seo.keywords = seo_request.keywords;
        }

        // Update the category
        match self.category_dao.update_category(&request.id, updated_category).await? {
            Some(category) => Ok(self.category_to_response(category)),
            None => Err("Failed to update category".into()),
        }
    }

    /// Delete a category
    pub async fn delete_category(&self, id: &str) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        debug!("Deleting category with ID: {}", id);

        if id.is_empty() {
            return Err("Category ID is required".into());
        }

        // Check if category exists
        let _existing = self.category_dao.get_category(id).await?
            .ok_or("Category not found")?;

        // Check if category has children
        let children = self.category_dao.get_children(id).await?;
        if !children.is_empty() {
            return Err("Cannot delete category with children".into());
        }

        // TODO: Check if category has products assigned
        // This would require integration with product service
        
        self.category_dao.delete_category(id).await
    }

    /// Helper method to convert Category model to CategoryResponse
    fn category_to_response(&self, category: Category) -> CategoryResponse {
        CategoryResponse {
            id: category.id.unwrap_or_default(),
            slug: category.slug,
            name: category.name,
            short_description: category.short_description,
            full_description: category.full_description,
            path: category.path,
            ancestors: category.ancestors,
            parent_id: category.parent_id,
            level: category.level,
            children_count: category.children_count,
            product_count: category.product_count,
            is_active: category.is_active,
            display_order: category.display_order,
            seo: Some(crate::catalog_messages::CategorySeo {
                meta_title: category.seo.meta_title,
                meta_description: category.seo.meta_description,
                keywords: category.seo.keywords,
            }),
            created_at: Some(prost_types::Timestamp {
                seconds: category.created_at.timestamp(),
                nanos: category.created_at.timestamp_subsec_nanos() as i32,
            }),
            updated_at: Some(prost_types::Timestamp {
                seconds: category.updated_at.timestamp(),
                nanos: category.updated_at.timestamp_subsec_nanos() as i32,
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::persistence::category_dao::CategoryDaoImpl;
    use mongodb::{Collection, Client as MongoClient};

    // Note: These tests would require a MongoDB test instance
    // For now, they serve as documentation of expected behavior

    #[tokio::test]
    #[ignore] // Ignore until we have test MongoDB setup
    async fn test_create_category_service() {
        // This test would verify the category service
        // properly validates input and creates categories
    }

    #[tokio::test]
    #[ignore]
    async fn test_duplicate_slug_validation() {
        // This test would verify that attempting to create
        // a category with an existing slug fails appropriately
    }
}
