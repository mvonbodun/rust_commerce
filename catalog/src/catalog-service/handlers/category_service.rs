use std::sync::Arc;
use crate::{
    model::{Category, CategorySeo},
    persistence::category_dao::CategoryDao,
    catalog_messages::{CreateCategoryRequest, CategoryResponse},
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
