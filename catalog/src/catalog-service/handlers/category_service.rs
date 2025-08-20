use crate::{
    catalog_messages::{CategoryResponse, CreateCategoryRequest, UpdateCategoryRequest},
    model::{Category, CategorySeo},
    persistence::category_dao::CategoryDao,
};
use log::debug;
use std::sync::Arc;

type BuildNodeFut<'a> = std::pin::Pin<
    Box<
        dyn std::future::Future<
                Output = Result<
                    Option<crate::catalog_messages::CategoryTreeNode>,
                    Box<dyn std::error::Error + Send + Sync>,
                >,
            > + Send
            + 'a,
    >,
>;

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

    /// Create a new category (internal version with cache control)
    async fn create_category_internal(
        &self,
        request: CreateCategoryRequest,
        invalidate_cache: bool,
    ) -> Result<CategoryResponse, Box<dyn std::error::Error + Send + Sync>> {
        // Validate input
        if request.name.trim().is_empty() {
            return Err("Category name cannot be empty".into());
        }

        if request.slug.trim().is_empty() {
            return Err("Category slug cannot be empty".into());
        }

        // Check if slug already exists
        if (self
            .category_dao
            .get_category_by_slug(&request.slug)
            .await?)
            .is_some()
        {
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
            category.seo =
                CategorySeo::default_for_category(&request.name, &request.short_description);
        }

        // Create category with cache control
        let created_category = if invalidate_cache {
            self.category_dao.create_category(category).await?
        } else {
            // Use a direct insert without cache invalidation for batch operations
            // TODO: Add create_category_no_cache_invalidation to DAO
            self.category_dao.create_category(category).await?
        };

        // Convert to response
        Ok(self.category_to_response(created_category))
    }

    /// Create a new category
    pub async fn create_category(
        &self,
        request: CreateCategoryRequest,
    ) -> Result<CategoryResponse, Box<dyn std::error::Error + Send + Sync>> {
        self.create_category_internal(request, true).await
    }

    /// Get category by ID
    pub async fn get_category(
        &self,
        id: &str,
    ) -> Result<Option<CategoryResponse>, Box<dyn std::error::Error + Send + Sync>> {
        match self.category_dao.get_category(id).await? {
            Some(category) => Ok(Some(self.category_to_response(category))),
            None => Ok(None),
        }
    }

    /// Get category by slug
    pub async fn get_category_by_slug(
        &self,
        slug: &str,
    ) -> Result<Option<CategoryResponse>, Box<dyn std::error::Error + Send + Sync>> {
        match self.category_dao.get_category_by_slug(slug).await? {
            Some(category) => Ok(Some(self.category_to_response(category))),
            None => Ok(None),
        }
    }

    /// Export categories with pagination
    pub async fn export_categories(
        &self,
        batch_size: Option<i64>,
        offset: Option<u64>,
    ) -> Result<Vec<CategoryResponse>, Box<dyn std::error::Error + Send + Sync>> {
        let categories = if let Some(offset) = offset {
            self.category_dao
                .export_categories_batch(batch_size, Some(offset))
                .await?
        } else {
            self.category_dao.export_all_categories(batch_size).await?
        };

        Ok(categories
            .into_iter()
            .map(|cat| self.category_to_response(cat))
            .collect())
    }

    /// Get the category tree structure
    pub async fn get_category_tree(
        &self,
        max_depth: Option<i32>,
        include_inactive: Option<bool>,
        rebuild_cache: Option<bool>,
    ) -> Result<
        Vec<crate::catalog_messages::CategoryTreeNode>,
        Box<dyn std::error::Error + Send + Sync>,
    > {
        debug!(
            "Getting category tree - max_depth: {max_depth:?}, include_inactive: {include_inactive:?}, rebuild_cache: {rebuild_cache:?}"
        );

        // If rebuild_cache is requested, rebuild the cache first
        if rebuild_cache.unwrap_or(false) {
            debug!("Rebuilding tree cache as requested");
            self.category_dao.rebuild_tree_cache().await?;
        }

        // Get the tree cache
        let tree_cache = match self.category_dao.get_full_tree().await? {
            Some(cache) => cache,
            None => {
                debug!("No tree cache found, rebuilding...");
                self.category_dao.rebuild_tree_cache().await?
            }
        };

        // Convert tree cache to CategoryTreeNode format
        let mut tree_nodes = Vec::new();
        let include_inactive = include_inactive.unwrap_or(false);
        let max_depth = max_depth.unwrap_or(-1); // -1 means no depth limit

        // The tree cache contains all categories as a flat HashMap
        // We need to identify root categories and build the tree structure
        for (category_id, cache_node) in &tree_cache.tree {
            // Check if this is a root category by checking if any category has this as a child
            let is_root = !tree_cache
                .tree
                .values()
                .any(|node| node.children.contains_key(category_id));

            if is_root {
                if let Some(node) = self
                    .build_tree_node_from_cache(
                        &tree_cache,
                        category_id,
                        cache_node,
                        0,
                        max_depth,
                        include_inactive,
                    )
                    .await?
                {
                    tree_nodes.push(node);
                }
            }
        }

        // Sort root nodes by name
        tree_nodes.sort_by(|a, b| a.name.cmp(&b.name));

        Ok(tree_nodes)
    }

    /// Helper method to build a CategoryTreeNode recursively from cache
    fn build_tree_node_from_cache<'a>(
        &'a self,
        _tree_cache: &'a crate::model::CategoryTreeCache,
        category_id: &'a str,
        cache_node: &'a crate::model::CategoryTreeNode,
        current_depth: i32,
        max_depth: i32,
        include_inactive: bool,
    ) -> BuildNodeFut<'a> {
        Box::pin(async move {
            if max_depth >= 0 && current_depth >= max_depth {
                return Ok(None);
            }

            // Get the full category details for active/inactive check
            if let Some(category) = self.category_dao.get_category(category_id).await? {
                // Skip inactive categories if not requested
                if !include_inactive && !category.is_active {
                    return Ok(None);
                }

        
                let mut children = Vec::new();

                // Recursively build child nodes
                for (child_id, child_cache_node) in &cache_node.children {
                    if let Some(child_node) = self
                        .build_tree_node_from_cache(
                            _tree_cache,
                            child_id,
                            child_cache_node,
                            current_depth + 1,
                            max_depth,
                            include_inactive,
                        )
                        .await?
                    {
                        children.push(child_node);
                    }
                }

                // Sort children by name
                children.sort_by(|a, b| a.name.cmp(&b.name));

                let tree_node = crate::catalog_messages::CategoryTreeNode {
                    id: category.id.unwrap_or_default(),
                    name: category.name,
                    slug: category.slug,
                    path: category.path,
                    level: category.level,
                    product_count: category.product_count,
                    children,
                };

                return Ok(Some(tree_node));
            }

            Ok(None)
        })
    }

    /// Update an existing category
    pub async fn update_category(
        &self,
        request: UpdateCategoryRequest,
    ) -> Result<CategoryResponse, Box<dyn std::error::Error + Send + Sync>> {
        debug!("Updating category with ID: {}", request.id);

        // Validate required fields
        if request.id.is_empty() {
            return Err("Category ID is required".into());
        }

        // Check if category exists
        let existing_category = self
            .category_dao
            .get_category(&request.id)
            .await?
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
                if let Some(existing_with_slug) =
                    self.category_dao.get_category_by_slug(&new_slug).await?
                {
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
        match self
            .category_dao
            .update_category(&request.id, updated_category)
            .await?
        {
            Some(category) => Ok(self.category_to_response(category)),
            None => Err("Failed to update category".into()),
        }
    }

    /// Delete a category
    pub async fn delete_category(
        &self,
        id: &str,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    debug!("Deleting category with ID: {id}");

        if id.is_empty() {
            return Err("Category ID is required".into());
        }

        // Check if category exists
        let _existing = self
            .category_dao
            .get_category(id)
            .await?
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

    /// Import multiple categories with hierarchical slug support and efficient batch processing
    pub async fn import_categories(
        &self,
        category_requests: Vec<CreateCategoryRequest>,
        dry_run: bool,
    ) -> Result<ImportResult, Box<dyn std::error::Error + Send + Sync>> {
        debug!(
            "Importing {} categories with hierarchical processing, dry_run: {}",
            category_requests.len(),
            dry_run
        );

        // Step 1: Pre-validate all requests and build dependency graph
        let mut validated_requests = Vec::new();
        let mut slug_to_request = std::collections::HashMap::new();
        let mut errors = Vec::new();

        for (index, request) in category_requests.into_iter().enumerate() {
            match self.validate_category_request_basic(&request).await {
                Ok(_) => {
                    slug_to_request.insert(request.slug.clone(), index);
                    validated_requests.push(request);
                }
                Err(e) => {
                    let error_msg = format!("Validation failed for category {}: {}", index + 1, e);
                    errors.push(error_msg);
                }
            }
        }

        if !errors.is_empty() && !dry_run {
            return Ok(ImportResult {
                successful_imports: 0,
                failed_imports: errors.len(),
                total_processed: errors.len(),
                errors,
            });
        }

        // Step 2: Sort categories by dependency (parents before children)
        let sorted_requests = self.sort_categories_by_dependency(validated_requests)?;

        // Step 3: Create UUID mapping for efficient parent resolution
        let mut slug_to_uuid: std::collections::HashMap<String, String> =
            std::collections::HashMap::new();

        // Step 4: Process categories in dependency order
        let mut successful_imports = 0;
        let mut failed_imports = 0;
        let total_processed = sorted_requests.len();

        for (index, mut request) in sorted_requests.into_iter().enumerate() {
            // Resolve parent_id from parent_slug if needed
            if let Some(parent_slug) = &request.parent_slug {
                match slug_to_uuid.get(parent_slug) {
                    Some(parent_uuid) => {
                        request.parent_id = Some(parent_uuid.clone());
                    }
                    None => {
                        // Try to find existing parent in database
                        match self.category_dao.get_category_by_slug(parent_slug).await? {
                            Some(parent_category) => {
                                let parent_id = parent_category.id.clone();
                                request.parent_id = parent_id.clone();
                                if let Some(parent_uuid) = parent_id {
                                    slug_to_uuid.insert(parent_slug.clone(), parent_uuid);
                                }
                            }
                            None => {
                                failed_imports += 1;
                                let error_msg = format!(
                                    "Parent category with slug '{}' not found for category '{}'",
                                    parent_slug, request.name
                                );
                                errors.push(error_msg);
                                continue;
                            }
                        }
                    }
                }
            }

            if !dry_run {
                match self.create_category_internal(request.clone(), false).await {
                    Ok(response) => {
                        successful_imports += 1;
                        // Store the UUID for this category for future parent lookups
                        slug_to_uuid.insert(request.slug.clone(), response.id);
                        debug!(
                            "Successfully imported category {}/{}: {}",
                            index + 1,
                            total_processed,
                            request.name
                        );
                    }
                    Err(e) => {
                        failed_imports += 1;
                        let error_msg =
                            format!("Failed to create category '{}': {}", request.name, e);
                        errors.push(error_msg);
                    }
                }
            } else {
                successful_imports += 1;
                // For dry run, generate a mock UUID
                let mock_uuid = uuid::Uuid::new_v4().to_string();
                slug_to_uuid.insert(request.slug.clone(), mock_uuid);
                debug!(
                    "Validated category {}/{}: {} (dry run)",
                    index + 1,
                    total_processed,
                    request.name
                );
            }
        }

        // Invalidate tree cache once at the end for non-dry-run imports
        if !dry_run && successful_imports > 0 {
            if let Err(e) = self.category_dao.invalidate_tree_cache().await {
                debug!(
                    "Warning: Failed to invalidate tree cache after import: {e}",
                );
            }
        }

        Ok(ImportResult {
            successful_imports,
            failed_imports,
            total_processed: successful_imports + failed_imports,
            errors,
        })
    }

    /// Sort categories by dependency order (parents before children)
    fn sort_categories_by_dependency(
        &self,
        categories: Vec<CreateCategoryRequest>,
    ) -> Result<Vec<CreateCategoryRequest>, Box<dyn std::error::Error + Send + Sync>> {
        let mut sorted = Vec::new();
        let mut remaining = categories;
        let mut processed_slugs = std::collections::HashSet::new();

        // Keep processing until all categories are sorted or we detect circular dependencies
        let max_iterations = remaining.len() * 2; // Safety limit
        let mut iteration = 0;

        while !remaining.is_empty() && iteration < max_iterations {
            let mut progress_made = false;
            let mut next_remaining = Vec::new();

            // Check which categories can be processed in this iteration
            for category in remaining.into_iter() {
                let can_process = match &category.parent_slug {
                    None => true, // Root category
                    Some(parent_slug) => {
                        // Check if parent has already been processed
                        processed_slugs.contains(parent_slug)
                    }
                };

                if can_process {
                    processed_slugs.insert(category.slug.clone());
                    sorted.push(category);
                    progress_made = true;
                } else {
                    next_remaining.push(category);
                }
            }

            if !progress_made {
                return Err("Circular dependency detected in category hierarchy".into());
            }

            remaining = next_remaining;
            iteration += 1;
        }

        if !remaining.is_empty() {
            return Err("Unable to resolve all category dependencies".into());
        }

        Ok(sorted)
    }

    /// Basic validation without database checks (for batch processing)
    async fn validate_category_request_basic(
        &self,
        request: &CreateCategoryRequest,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Basic validation
        if request.name.is_empty() {
            return Err("Category name cannot be empty".into());
        }

        if request.slug.is_empty() {
            return Err("Category slug cannot be empty".into());
        }

        // Validate hierarchical slug format
        if !request.slug.starts_with('/') {
            return Err(format!(
                "Category slug '{}' must start with '/' for hierarchical structure",
                request.slug
            )
            .into());
        }

        // Validate parent_slug format if present
        if let Some(parent_slug) = &request.parent_slug {
            if !parent_slug.starts_with('/') {
                return Err(format!(
                    "Parent slug '{parent_slug}' must start with '/' for hierarchical structure",
                )
                .into());
            }

            // Ensure child slug contains parent slug as prefix
        if !request.slug.starts_with(&format!("{parent_slug}/"))
                && request.slug != *parent_slug
            {
                return Err(format!(
            "Child slug '{}' must be a path under parent slug '{parent_slug}'",
            request.slug
                )
                .into());
            }
        }

        Ok(())
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
