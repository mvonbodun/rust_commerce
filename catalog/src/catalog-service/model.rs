use bson::oid::ObjectId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Product {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub name: String,
    pub long_description: Option<String>,
    pub brand: Option<String>,
    pub slug: Option<String>,
    pub product_ref: Option<i32>, // Changed from String to i32 to match sample
    pub product_type: Option<String>,
    pub seo_title: Option<String>,
    pub seo_description: Option<String>,
    pub seo_keywords: Option<String>,
    pub display_on_site: bool,
    pub tax_code: Option<String>,
    pub related_products: Vec<String>,
    pub reviews: Option<Reviews>,
    pub hierarchical_categories: Option<HierarchicalCategories>,
    pub list_categories: Vec<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub created_by: Option<String>,
    pub updated_by: Option<String>,
    pub defining_attributes: HashMap<String, String>,
    pub descriptive_attributes: HashMap<String, String>,
    pub default_variant: Option<String>,
    pub variants: Vec<ProductVariant>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Reviews {
    pub bayesian_avg: i32,
    pub count: i32,
    pub rating: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HierarchicalCategories {
    pub lvl0: Option<String>,
    pub lvl1: Option<String>,
    pub lvl2: Option<String>,
    pub lvl3: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProductVariant {
    pub sku: String,
    pub defining_attributes: Option<HashMap<String, String>>, // Made optional as some variants don't have it
    pub abbreviated_color: Option<String>,
    pub abbreviated_size: Option<String>, // Added field from sample data
    pub height: Option<f64>, // Added dimension fields
    pub width: Option<f64>,
    pub length: Option<f64>,
    pub weight: Option<f64>,
    pub weight_unit: Option<String>,
    pub packaging: Option<Packaging>, // Added packaging information
    pub image_urls: Vec<String>, // Changed from single image to array of URLs
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Packaging {
    pub height: Option<f64>,
    pub width: Option<f64>,
    pub length: Option<f64>,
    pub weight: Option<f64>,
    pub weight_unit: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_deserialize_sample_product() {
        // Read the sample product data
        let sample_data = fs::read_to_string("sample_records_backup/sample_product_mongo_record.json")
            .expect("Failed to read sample product file");
        
        // Try to deserialize it into our Product model
        let product: Result<Product, _> = serde_json::from_str(&sample_data);
        
        match product {
            Ok(p) => {
                println!("Successfully deserialized product: {}", p.name);
                assert_eq!(p.name, "Calvin Klein Performance Ripstop Cargo Shorts");
                assert_eq!(p.brand, Some("Calvin Klein Performance".to_string()));
                assert_eq!(p.product_ref, Some(320));
                assert!(!p.variants.is_empty());
                
                // Check first variant
                let first_variant = &p.variants[0];
                assert_eq!(first_variant.sku, "0096234260");
                assert_eq!(first_variant.abbreviated_color, Some("GRAY".to_string()));
                assert!(first_variant.image_urls.len() > 0);
                assert!(first_variant.packaging.is_some());
            }
            Err(e) => {
                panic!("Failed to deserialize sample product: {}", e);
            }
        }
    }
}
