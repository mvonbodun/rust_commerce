use async_trait::async_trait;
use mongodb::{Collection, bson::doc};
use crate::model::Product;
use std::error::Error;
use futures::TryStreamExt;

#[async_trait]
pub trait ProductDao {
    async fn create_product(&self, product: Product) -> Result<Product, Box<dyn Error + Send + Sync>>;
    async fn get_product(&self, id: &str) -> Result<Option<Product>, Box<dyn Error + Send + Sync>>;
    async fn get_product_by_slug(&self, slug: &str) -> Result<Option<Product>, Box<dyn Error + Send + Sync>>;
    async fn update_product(&self, id: &str, product: Product) -> Result<Option<Product>, Box<dyn Error + Send + Sync>>;
    async fn delete_product(&self, id: &str) -> Result<bool, Box<dyn Error + Send + Sync>>;
    async fn search_products(&self, query: Option<&str>, categories: &[String], brand: Option<&str>, limit: Option<i64>, offset: Option<u64>) -> Result<Vec<Product>, Box<dyn Error + Send + Sync>>;
    async fn export_all_products(&self, batch_size: Option<i64>) -> Result<Vec<Product>, Box<dyn Error + Send + Sync>>;
    async fn export_products_batch(&self, batch_size: Option<i64>, offset: Option<u64>) -> Result<Vec<Product>, Box<dyn Error + Send + Sync>>;
}

pub struct ProductDaoImpl {
    collection: Collection<Product>,
}

impl ProductDaoImpl {
    pub fn new(collection: Collection<Product>) -> Self {
        Self { collection }
    }
}

#[async_trait]
impl ProductDao for ProductDaoImpl {
    async fn create_product(&self, mut product: Product) -> Result<Product, Box<dyn Error + Send + Sync>> {
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

    async fn get_product_by_slug(&self, slug: &str) -> Result<Option<Product>, Box<dyn Error + Send + Sync>> {
        let product = self.collection.find_one(doc! { "slug": &slug }).await?;
        Ok(product)
    }

    async fn update_product(&self, id: &str, product: Product) -> Result<Option<Product>, Box<dyn Error + Send + Sync>> {
        let result = self.collection.replace_one(
            doc! { "_id": &id },
            &product,
        ).await?;
        
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

    async fn search_products(&self, query: Option<&str>, categories: &[String], brand: Option<&str>, limit: Option<i64>, offset: Option<u64>) -> Result<Vec<Product>, Box<dyn Error + Send + Sync>> {
        let mut filter = doc! {};
        
        if let Some(q) = query {
            filter.insert("$or", vec![
                doc! { "name": { "$regex": q, "$options": "i" } },
                doc! { "long_description": { "$regex": q, "$options": "i" } },
                doc! { "seo_keywords": { "$regex": q, "$options": "i" } }
            ]);
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

    async fn export_all_products(&self, batch_size: Option<i64>) -> Result<Vec<Product>, Box<dyn Error + Send + Sync>> {
        // Use a much smaller batch size to avoid NATS payload limits
        // NATS has a default max payload of 1MB, so we need to be conservative
        let batch_size = batch_size.unwrap_or(50); // Reduced default batch size to 50
        
        let cursor = self.collection
            .find(doc! {})
            .limit(batch_size)
            .await?;
        
        let products: Vec<Product> = cursor.try_collect().await?;
        
        Ok(products)
    }

    async fn export_products_batch(&self, batch_size: Option<i64>, offset: Option<u64>) -> Result<Vec<Product>, Box<dyn Error + Send + Sync>> {
        let batch_size = batch_size.unwrap_or(50); // Conservative batch size
        let offset = offset.unwrap_or(0);
        
        let cursor = self.collection
            .find(doc! {})
            .limit(batch_size)
            .skip(offset)
            .await?;
        
        let products: Vec<Product> = cursor.try_collect().await?;
        
        Ok(products)
    }
}
