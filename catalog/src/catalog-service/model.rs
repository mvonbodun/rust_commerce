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
    pub product_ref: Option<String>,
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
    pub defining_attributes: HashMap<String, String>,
    pub abbreviated_color: Option<String>,
    pub upc: Option<String>,
    pub inventory: Option<Inventory>,
    pub pricing: Option<Pricing>,
    pub image: Option<Image>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Inventory {
    pub quantity_available: i32,
    pub quantity_reserved: i32,
    pub status: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Pricing {
    pub list_price: Option<Price>,
    pub sale_price: Option<Price>,
    pub msrp: Option<Price>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Price {
    pub amount: String,
    pub currency: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Image {
    pub url: String,
    pub alt_text: Option<String>,
    pub width: Option<i32>,
    pub height: Option<i32>,
}
