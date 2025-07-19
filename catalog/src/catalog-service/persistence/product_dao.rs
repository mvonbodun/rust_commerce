use async_trait::async_trait;
use bson::oid::ObjectId;
use mongodb::{Collection, bson::doc};
use crate::model::Product;
use std::error::Error;
use futures::TryStreamExt;

#[async_trait]
pub trait ProductDao {
    async fn create_product(&self, product: Product) -> Result<Product, Box<dyn Error + Send + Sync>>;
    async fn get_product(&self, id: &str) -> Result<Option<Product>, Box<dyn Error + Send + Sync>>;
    async fn update_product(&self, id: &str, product: Product) -> Result<Option<Product>, Box<dyn Error + Send + Sync>>;
    async fn delete_product(&self, id: &str) -> Result<bool, Box<dyn Error + Send + Sync>>;
    async fn search_products(&self, query: Option<&str>, categories: &[String], brand: Option<&str>, limit: Option<i64>, offset: Option<u64>) -> Result<Vec<Product>, Box<dyn Error + Send + Sync>>;
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
        product.id = Some(ObjectId::new());
        let result = self.collection.insert_one(&product).await?;
        product.id = Some(result.inserted_id.as_object_id().unwrap());
        Ok(product)
    }

    async fn get_product(&self, id: &str) -> Result<Option<Product>, Box<dyn Error + Send + Sync>> {
        let object_id = ObjectId::parse_str(id)?;
        let product = self.collection.find_one(doc! { "_id": object_id }).await?;
        Ok(product)
    }

    async fn update_product(&self, id: &str, mut product: Product) -> Result<Option<Product>, Box<dyn Error + Send + Sync>> {
        let object_id = ObjectId::parse_str(id)?;
        product.id = Some(object_id);
        
        let result = self.collection.replace_one(
            doc! { "_id": object_id },
            &product,
        ).await?;
        
        if result.modified_count > 0 {
            Ok(Some(product))
        } else {
            Ok(None)
        }
    }

    async fn delete_product(&self, id: &str) -> Result<bool, Box<dyn Error + Send + Sync>> {
        let object_id = ObjectId::parse_str(id)?;
        let result = self.collection.delete_one(doc! { "_id": object_id }).await?;
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
}
