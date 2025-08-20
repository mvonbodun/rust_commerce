use crate::model::{Product, ProductSlug};
use async_trait::async_trait;
use base64::{engine::general_purpose, Engine as _};
use futures::TryStreamExt;
use mongodb::{bson::doc, Collection, Database};
use std::error::Error;

#[async_trait]
pub trait ProductDao {
    async fn create_product(
        &self,
        product: Product,
    ) -> Result<Product, Box<dyn Error + Send + Sync>>;
    async fn get_product(&self, id: &str) -> Result<Option<Product>, Box<dyn Error + Send + Sync>>;
    async fn get_product_by_slug(
        &self,
        slug: &str,
    ) -> Result<Option<Product>, Box<dyn Error + Send + Sync>>;
    async fn update_product(
        &self,
        id: &str,
        product: Product,
    ) -> Result<Option<Product>, Box<dyn Error + Send + Sync>>;
    async fn delete_product(&self, id: &str) -> Result<bool, Box<dyn Error + Send + Sync>>;
    async fn search_products(
        &self,
        query: Option<&str>,
        categories: &[String],
        brand: Option<&str>,
        limit: Option<i64>,
        offset: Option<u64>,
    ) -> Result<Vec<Product>, Box<dyn Error + Send + Sync>>;
    async fn export_all_products(
        &self,
        batch_size: Option<i64>,
    ) -> Result<Vec<Product>, Box<dyn Error + Send + Sync>>;
    async fn export_products_batch(
        &self,
        batch_size: Option<i64>,
        offset: Option<u64>,
    ) -> Result<Vec<Product>, Box<dyn Error + Send + Sync>>;
    async fn get_product_slugs_paginated(
        &self,
        batch_size: i32,
        cursor: Option<String>,
        include_inactive: bool,
    ) -> Result<(Vec<String>, Option<String>, bool), Box<dyn Error + Send + Sync>>;
}

pub struct ProductDaoImpl {
    collection: Collection<Product>,
    db: Database,
}

impl ProductDaoImpl {
    pub fn new(collection: Collection<Product>, db: Database) -> Self {
        Self { collection, db }
    }
}

#[async_trait]
impl ProductDao for ProductDaoImpl {
    async fn create_product(
        &self,
        mut product: Product,
    ) -> Result<Product, Box<dyn Error + Send + Sync>> {
        let result = self.collection.insert_one(&product).await?;
        // The inserted_id should match what we set, but let's be safe
        if let Some(inserted_object_id) = result.inserted_id.as_object_id() {
            product.id = Some(inserted_object_id.to_hex());
        }
        Ok(product)
    }

    async fn get_product(&self, id: &str) -> Result<Option<Product>, Box<dyn Error + Send + Sync>> {
        let product = self.collection.find_one(doc! { "_id": &id }).await?;
        Ok(product)
    }

    async fn get_product_by_slug(
        &self,
        slug: &str,
    ) -> Result<Option<Product>, Box<dyn Error + Send + Sync>> {
        let product = self.collection.find_one(doc! { "slug": &slug }).await?;
        Ok(product)
    }

    async fn update_product(
        &self,
        id: &str,
        product: Product,
    ) -> Result<Option<Product>, Box<dyn Error + Send + Sync>> {
        let result = self
            .collection
            .replace_one(doc! { "_id": &id }, &product)
            .await?;

        if result.modified_count > 0 {
            Ok(Some(product))
        } else {
            Ok(None)
        }
    }

    async fn delete_product(&self, id: &str) -> Result<bool, Box<dyn Error + Send + Sync>> {
        let result = self.collection.delete_one(doc! { "_id": &id }).await?;
        Ok(result.deleted_count > 0)
    }

    async fn search_products(
        &self,
        query: Option<&str>,
        categories: &[String],
        brand: Option<&str>,
        limit: Option<i64>,
        offset: Option<u64>,
    ) -> Result<Vec<Product>, Box<dyn Error + Send + Sync>> {
        let mut filter = doc! {};

        if let Some(q) = query {
            filter.insert(
                "$or",
                vec![
                    doc! { "name": { "$regex": q, "$options": "i" } },
                    doc! { "long_description": { "$regex": q, "$options": "i" } },
                    doc! { "seo_keywords": { "$regex": q, "$options": "i" } },
                ],
            );
        }

        if !categories.is_empty() {
            filter.insert("list_categories", doc! { "$in": categories });
        }

        if let Some(b) = brand {
            filter.insert("brand", b);
        }

        let mut find_options = self.collection.find(filter);

        if let Some(l) = limit {
            find_options = find_options.limit(l);
        }
        if let Some(o) = offset {
            find_options = find_options.skip(o);
        }

        let cursor = find_options.await?;
        let products: Vec<Product> = cursor.try_collect().await?;

        Ok(products)
    }

    async fn export_all_products(
        &self,
        batch_size: Option<i64>,
    ) -> Result<Vec<Product>, Box<dyn Error + Send + Sync>> {
        // Use a much smaller batch size to avoid NATS payload limits
        // NATS has a default max payload of 1MB, so we need to be conservative
        let batch_size = batch_size.unwrap_or(50); // Reduced default batch size to 50

        let cursor = self.collection.find(doc! {}).limit(batch_size).await?;

        let products: Vec<Product> = cursor.try_collect().await?;

        Ok(products)
    }

    async fn export_products_batch(
        &self,
        batch_size: Option<i64>,
        offset: Option<u64>,
    ) -> Result<Vec<Product>, Box<dyn Error + Send + Sync>> {
        let batch_size = batch_size.unwrap_or(50); // Conservative batch size
        let offset = offset.unwrap_or(0);

        let cursor = self
            .collection
            .find(doc! {})
            .limit(batch_size)
            .skip(offset)
            .await?;

        let products: Vec<Product> = cursor.try_collect().await?;

        Ok(products)
    }

    async fn get_product_slugs_paginated(
        &self,
        batch_size: i32,
        cursor: Option<String>,
        include_inactive: bool,
    ) -> Result<(Vec<String>, Option<String>, bool), Box<dyn Error + Send + Sync>> {
        // Validate and clamp batch_size
    let batch_size = batch_size.clamp(10, 1000);

        // Build the query filter
        let mut query = doc! {};

        // Add cursor condition if provided
        if let Some(cursor_slug) = &cursor {
            // Decode base64 cursor to get the last slug
            let decoded_cursor = match general_purpose::STANDARD.decode(cursor_slug) {
                Ok(decoded) => match String::from_utf8(decoded) {
                    Ok(slug) => slug,
                    Err(_) => return Err("Invalid cursor format".into()),
                },
                Err(_) => return Err("Invalid cursor encoding".into()),
            };
            query.insert("slug", doc! { "$gt": decoded_cursor });
        }

        // Add active filter if needed
        if !include_inactive {
            query.insert("display_on_site", true);
        }

        // Create a ProductSlug collection for efficient querying
        let slug_collection: Collection<ProductSlug> = self.db.collection("products");

        // Execute query with typed ProductSlug collection
        let mut cursor = slug_collection
            .find(query)
            .projection(doc! { "slug": 1, "_id": 0 })
            .sort(doc! { "slug": 1 })
            .limit(batch_size as i64 + 1) // Fetch one extra to check if more results exist
            .await?;

        // Collect results as ProductSlug structs
        let mut product_slugs = Vec::new();
        while let Some(product_slug) = cursor.try_next().await? {
            product_slugs.push(product_slug);
        }

        // Extract slugs
        let total_count = product_slugs.len();
        let items_to_return = std::cmp::min(batch_size as usize, total_count);
        let slugs: Vec<String> = product_slugs
            .iter()
            .take(items_to_return)
            .map(|ps| ps.slug.clone())
            .collect();

        // Check if there are more results
        let has_more = total_count > batch_size as usize;
        let next_cursor = if has_more && !slugs.is_empty() {
            let last_slug = slugs.last().unwrap();
            Some(general_purpose::STANDARD.encode(last_slug))
        } else {
            None
        };

        Ok((slugs, next_cursor, has_more))
    }
}
