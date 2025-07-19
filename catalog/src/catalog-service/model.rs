use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

/// A type-safe wrapper for Bayesian average ratings that ensures values are always rounded to one decimal place.
/// This prevents precision issues and enforces business rules for rating calculations.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BayesianAverage(Decimal);

impl BayesianAverage {
    /// Creates a new BayesianAverage, automatically rounding to one decimal place
    pub fn new(value: f32) -> Self {
        let decimal = Decimal::from_f32_retain(value).unwrap_or(dec!(0.0));
        Self(decimal.round_dp(1))
    }
    
    /// Returns the value as an f32 for compatibility with existing code
    pub fn as_f32(&self) -> f32 {
        f32::try_from(self.0).unwrap_or(0.0)
    }
    
    /// Returns the value as a Decimal for precise calculations
    pub fn as_decimal(&self) -> Decimal {
        self.0
    }
}

impl From<f32> for BayesianAverage {
    fn from(value: f32) -> Self {
        Self::new(value)
    }
}

impl From<BayesianAverage> for f32 {
    fn from(avg: BayesianAverage) -> Self {
        avg.as_f32()
    }
}

// Custom serialization to ensure we always serialize as f32 for JSON compatibility
impl Serialize for BayesianAverage {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_f32(self.as_f32())
    }
}

// Custom deserialization that automatically rounds to one decimal place
impl<'de> Deserialize<'de> for BayesianAverage {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = f32::deserialize(deserializer)?;
        Ok(BayesianAverage::new(value))
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Product {
    #[serde(rename = "_id")]
    pub id: Option<String>,
    pub name: String,
    pub long_description: Option<String>,
    pub brand: Option<String>,
    pub slug: Option<String>,
    pub product_ref: String, // Changed back to String
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
    pub bayesian_avg: BayesianAverage,
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

impl Product {
    pub fn builder() -> ProductBuilder {
        ProductBuilder::default()
    }
}

#[derive(Default)]
pub struct ProductBuilder {
    id: Option<String>,
    name: String,
    long_description: Option<String>,
    brand: Option<String>,
    slug: Option<String>,
    product_ref: String,
    product_type: Option<String>,
    seo_title: Option<String>,
    seo_description: Option<String>,
    seo_keywords: Option<String>,
    display_on_site: bool,
    tax_code: Option<String>,
    related_products: Vec<String>,
    reviews: Option<Reviews>,
    hierarchical_categories: Option<HierarchicalCategories>,
    list_categories: Vec<String>,
    created_at: Option<DateTime<Utc>>,
    updated_at: Option<DateTime<Utc>>,
    created_by: Option<String>,
    updated_by: Option<String>,
    defining_attributes: HashMap<String, String>,
    descriptive_attributes: HashMap<String, String>,
    default_variant: Option<String>,
    variants: Vec<ProductVariant>,
}

impl ProductBuilder {
    pub fn new(name: String, product_ref: String) -> ProductBuilder {
        ProductBuilder {
            id: Some(Uuid::new_v4().to_string()),
            name,
            long_description: None,
            brand: None,
            slug: None,
            product_ref,
            product_type: None,
            seo_title: None,
            seo_description: None,
            seo_keywords: None,
            display_on_site: true, // Default to true
            tax_code: None,
            related_products: Vec::new(),
            reviews: None,
            hierarchical_categories: None,
            list_categories: Vec::new(),
            created_at: Some(Utc::now()),
            updated_at: Some(Utc::now()),
            created_by: None,
            updated_by: None,
            defining_attributes: HashMap::new(),
            descriptive_attributes: HashMap::new(),
            default_variant: None,
            variants: Vec::new(),
        }
    }

    pub fn long_description(&mut self, long_description: String) -> &mut Self {
        self.long_description = Some(long_description);
        self
    }

    pub fn brand(&mut self, brand: String) -> &mut Self {
        self.brand = Some(brand);
        self
    }

    pub fn slug(&mut self, slug: String) -> &mut Self {
        self.slug = Some(slug);
        self
    }

    pub fn product_type(&mut self, product_type: String) -> &mut Self {
        self.product_type = Some(product_type);
        self
    }

    pub fn seo_title(&mut self, seo_title: String) -> &mut Self {
        self.seo_title = Some(seo_title);
        self
    }

    pub fn seo_description(&mut self, seo_description: String) -> &mut Self {
        self.seo_description = Some(seo_description);
        self
    }

    pub fn seo_keywords(&mut self, seo_keywords: String) -> &mut Self {
        self.seo_keywords = Some(seo_keywords);
        self
    }

    pub fn display_on_site(&mut self, display_on_site: bool) -> &mut Self {
        self.display_on_site = display_on_site;
        self
    }

    pub fn tax_code(&mut self, tax_code: String) -> &mut Self {
        self.tax_code = Some(tax_code);
        self
    }

    pub fn related_products(&mut self, related_products: Vec<String>) -> &mut Self {
        self.related_products = related_products;
        self
    }

    pub fn add_related_product(&mut self, product_id: String) -> &mut Self {
        self.related_products.push(product_id);
        self
    }

    pub fn reviews(&mut self, reviews: Reviews) -> &mut Self {
        self.reviews = Some(reviews);
        self
    }

    pub fn hierarchical_categories(&mut self, hierarchical_categories: HierarchicalCategories) -> &mut Self {
        self.hierarchical_categories = Some(hierarchical_categories);
        self
    }

    pub fn list_categories(&mut self, list_categories: Vec<String>) -> &mut Self {
        self.list_categories = list_categories;
        self
    }

    pub fn add_list_category(&mut self, category: String) -> &mut Self {
        self.list_categories.push(category);
        self
    }

    pub fn created_by(&mut self, created_by: String) -> &mut Self {
        self.created_by = Some(created_by);
        self
    }

    pub fn updated_by(&mut self, updated_by: String) -> &mut Self {
        self.updated_by = Some(updated_by);
        self
    }

    pub fn defining_attributes(&mut self, defining_attributes: HashMap<String, String>) -> &mut Self {
        self.defining_attributes = defining_attributes;
        self
    }

    pub fn add_defining_attribute(&mut self, key: String, value: String) -> &mut Self {
        self.defining_attributes.insert(key, value);
        self
    }

    pub fn descriptive_attributes(&mut self, descriptive_attributes: HashMap<String, String>) -> &mut Self {
        self.descriptive_attributes = descriptive_attributes;
        self
    }

    pub fn add_descriptive_attribute(&mut self, key: String, value: String) -> &mut Self {
        self.descriptive_attributes.insert(key, value);
        self
    }

    pub fn default_variant(&mut self, default_variant: String) -> &mut Self {
        self.default_variant = Some(default_variant);
        self
    }

    pub fn variants(&mut self, variants: Vec<ProductVariant>) -> &mut Self {
        self.variants = variants;
        self
    }

    pub fn add_variant(&mut self, variant: ProductVariant) -> &mut Self {
        self.variants.push(variant);
        self
    }

    pub fn build(&mut self) -> Product {
        Product {
            id: self.id.clone(),
            name: self.name.clone(),
            long_description: self.long_description.clone(),
            brand: self.brand.clone(),
            slug: self.slug.clone(),
            product_ref: self.product_ref.clone(),
            product_type: self.product_type.clone(),
            seo_title: self.seo_title.clone(),
            seo_description: self.seo_description.clone(),
            seo_keywords: self.seo_keywords.clone(),
            display_on_site: self.display_on_site,
            tax_code: self.tax_code.clone(),
            related_products: self.related_products.clone(),
            reviews: self.reviews.clone(),
            hierarchical_categories: self.hierarchical_categories.clone(),
            list_categories: self.list_categories.clone(),
            created_at: self.created_at,
            updated_at: self.updated_at,
            created_by: self.created_by.clone(),
            updated_by: self.updated_by.clone(),
            defining_attributes: self.defining_attributes.clone(),
            descriptive_attributes: self.descriptive_attributes.clone(),
            default_variant: self.default_variant.clone(),
            variants: self.variants.clone(),
        }
    }
}

impl ProductVariant {
    pub fn builder() -> ProductVariantBuilder {
        ProductVariantBuilder::default()
    }
}

#[derive(Default)]
pub struct ProductVariantBuilder {
    sku: String,
    defining_attributes: Option<HashMap<String, String>>,
    abbreviated_color: Option<String>,
    abbreviated_size: Option<String>,
    height: Option<f64>,
    width: Option<f64>,
    length: Option<f64>,
    weight: Option<f64>,
    weight_unit: Option<String>,
    packaging: Option<Packaging>,
    image_urls: Vec<String>,
}

impl ProductVariantBuilder {
    pub fn new(sku: String) -> ProductVariantBuilder {
        ProductVariantBuilder {
            sku,
            defining_attributes: None,
            abbreviated_color: None,
            abbreviated_size: None,
            height: None,
            width: None,
            length: None,
            weight: None,
            weight_unit: None,
            packaging: None,
            image_urls: Vec::new(),
        }
    }

    pub fn defining_attributes(&mut self, defining_attributes: HashMap<String, String>) -> &mut Self {
        self.defining_attributes = Some(defining_attributes);
        self
    }

    pub fn add_defining_attribute(&mut self, key: String, value: String) -> &mut Self {
        if self.defining_attributes.is_none() {
            self.defining_attributes = Some(HashMap::new());
        }
        if let Some(ref mut attrs) = self.defining_attributes {
            attrs.insert(key, value);
        }
        self
    }

    pub fn abbreviated_color(&mut self, abbreviated_color: String) -> &mut Self {
        self.abbreviated_color = Some(abbreviated_color);
        self
    }

    pub fn abbreviated_size(&mut self, abbreviated_size: String) -> &mut Self {
        self.abbreviated_size = Some(abbreviated_size);
        self
    }

    pub fn dimensions(&mut self, height: f64, width: f64, length: f64) -> &mut Self {
        self.height = Some(height);
        self.width = Some(width);
        self.length = Some(length);
        self
    }

    pub fn height(&mut self, height: f64) -> &mut Self {
        self.height = Some(height);
        self
    }

    pub fn width(&mut self, width: f64) -> &mut Self {
        self.width = Some(width);
        self
    }

    pub fn length(&mut self, length: f64) -> &mut Self {
        self.length = Some(length);
        self
    }

    pub fn weight(&mut self, weight: f64, unit: String) -> &mut Self {
        self.weight = Some(weight);
        self.weight_unit = Some(unit);
        self
    }

    pub fn packaging(&mut self, packaging: Packaging) -> &mut Self {
        self.packaging = Some(packaging);
        self
    }

    pub fn image_urls(&mut self, image_urls: Vec<String>) -> &mut Self {
        self.image_urls = image_urls;
        self
    }

    pub fn add_image_url(&mut self, image_url: String) -> &mut Self {
        self.image_urls.push(image_url);
        self
    }

    pub fn build(&mut self) -> ProductVariant {
        ProductVariant {
            sku: self.sku.clone(),
            defining_attributes: self.defining_attributes.clone(),
            abbreviated_color: self.abbreviated_color.clone(),
            abbreviated_size: self.abbreviated_size.clone(),
            height: self.height,
            width: self.width,
            length: self.length,
            weight: self.weight,
            weight_unit: self.weight_unit.clone(),
            packaging: self.packaging.clone(),
            image_urls: self.image_urls.clone(),
        }
    }
}

impl Packaging {
    pub fn builder() -> PackagingBuilder {
        PackagingBuilder::default()
    }
}

#[derive(Default)]
pub struct PackagingBuilder {
    height: Option<f64>,
    width: Option<f64>,
    length: Option<f64>,
    weight: Option<f64>,
    weight_unit: Option<String>,
}

impl PackagingBuilder {
    pub fn new() -> PackagingBuilder {
        PackagingBuilder::default()
    }

    pub fn dimensions(&mut self, height: f64, width: f64, length: f64) -> &mut Self {
        self.height = Some(height);
        self.width = Some(width);
        self.length = Some(length);
        self
    }

    pub fn height(&mut self, height: f64) -> &mut Self {
        self.height = Some(height);
        self
    }

    pub fn width(&mut self, width: f64) -> &mut Self {
        self.width = Some(width);
        self
    }

    pub fn length(&mut self, length: f64) -> &mut Self {
        self.length = Some(length);
        self
    }

    pub fn weight(&mut self, weight: f64, unit: String) -> &mut Self {
        self.weight = Some(weight);
        self.weight_unit = Some(unit);
        self
    }

    pub fn build(&mut self) -> Packaging {
        Packaging {
            height: self.height,
            width: self.width,
            length: self.length,
            weight: self.weight,
            weight_unit: self.weight_unit.clone(),
        }
    }
}

impl Reviews {
    pub fn builder() -> ReviewsBuilder {
        ReviewsBuilder::default()
    }
}

#[derive(Default)]
pub struct ReviewsBuilder {
    bayesian_avg: f32,
    count: i32,
    rating: i32,
}

impl ReviewsBuilder {
    pub fn new(bayesian_avg: f32, count: i32, rating: i32) -> ReviewsBuilder {
        ReviewsBuilder {
            bayesian_avg,
            count,
            rating,
        }
    }

    pub fn bayesian_avg(&mut self, bayesian_avg: f32) -> &mut Self {
        self.bayesian_avg = bayesian_avg;
        self
    }

    pub fn count(&mut self, count: i32) -> &mut Self {
        self.count = count;
        self
    }

    pub fn rating(&mut self, rating: i32) -> &mut Self {
        self.rating = rating;
        self
    }

    pub fn build(&mut self) -> Reviews {
        Reviews {
            bayesian_avg: BayesianAverage::new(self.bayesian_avg),
            count: self.count,
            rating: self.rating,
        }
    }
}

impl HierarchicalCategories {
    pub fn builder() -> HierarchicalCategoriesBuilder {
        HierarchicalCategoriesBuilder::default()
    }
}

#[derive(Default)]
pub struct HierarchicalCategoriesBuilder {
    lvl0: Option<String>,
    lvl1: Option<String>,
    lvl2: Option<String>,
    lvl3: Option<String>,
}

impl HierarchicalCategoriesBuilder {
    pub fn new() -> HierarchicalCategoriesBuilder {
        HierarchicalCategoriesBuilder::default()
    }

    pub fn lvl0(&mut self, lvl0: String) -> &mut Self {
        self.lvl0 = Some(lvl0);
        self
    }

    pub fn lvl1(&mut self, lvl1: String) -> &mut Self {
        self.lvl1 = Some(lvl1);
        self
    }

    pub fn lvl2(&mut self, lvl2: String) -> &mut Self {
        self.lvl2 = Some(lvl2);
        self
    }

    pub fn lvl3(&mut self, lvl3: String) -> &mut Self {
        self.lvl3 = Some(lvl3);
        self
    }

    pub fn build(&mut self) -> HierarchicalCategories {
        HierarchicalCategories {
            lvl0: self.lvl0.clone(),
            lvl1: self.lvl1.clone(),
            lvl2: self.lvl2.clone(),
            lvl3: self.lvl3.clone(),
        }
    }
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
                assert_eq!(p.product_ref, "P000223554");
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

    #[test]
    fn test_product_builder() {
        // Test the builder pattern for creating a product
        let product = ProductBuilder::new("Test Product".to_string(), "TEST001".to_string())
            .brand("Test Brand".to_string())
            .long_description("This is a test product".to_string())
            .seo_title("Test Product - Best Quality".to_string())
            .tax_code("TAX001".to_string())
            .add_defining_attribute("color".to_string(), "blue".to_string())
            .add_descriptive_attribute("material".to_string(), "cotton".to_string())
            .add_list_category("clothing".to_string())
            .add_list_category("shirts".to_string())
            .created_by("test_user".to_string())
            .build();

        // Verify the product was created correctly
        assert!(product.id.is_some());
        assert_eq!(product.name, "Test Product");
        assert_eq!(product.product_ref, "TEST001");
        assert_eq!(product.brand, Some("Test Brand".to_string()));
        assert_eq!(product.long_description, Some("This is a test product".to_string()));
        assert_eq!(product.seo_title, Some("Test Product - Best Quality".to_string()));
        assert_eq!(product.tax_code, Some("TAX001".to_string()));
        assert_eq!(product.display_on_site, true);
        assert!(product.created_at.is_some());
        assert!(product.updated_at.is_some());
        assert_eq!(product.created_by, Some("test_user".to_string()));
        
        // Check attributes
        assert_eq!(product.defining_attributes.get("color"), Some(&"blue".to_string()));
        assert_eq!(product.descriptive_attributes.get("material"), Some(&"cotton".to_string()));
        
        // Check categories
        assert_eq!(product.list_categories.len(), 2);
        assert!(product.list_categories.contains(&"clothing".to_string()));
        assert!(product.list_categories.contains(&"shirts".to_string()));

        // Verify UUID format for id
        if let Some(id) = &product.id {
            assert_eq!(id.len(), 36); // UUID string length
            assert!(id.contains('-')); // UUID contains dashes
        }
    }

    #[test]
    fn test_variant_builder() {
        // Test the builder pattern for creating a product variant
        let variant = ProductVariantBuilder::new("SKU123".to_string())
            .abbreviated_color("RED".to_string())
            .abbreviated_size("M".to_string())
            .dimensions(10.0, 5.0, 2.0)
            .weight(0.5, "kg".to_string())
            .add_defining_attribute("size".to_string(), "Medium".to_string())
            .add_image_url("https://example.com/image1.jpg".to_string())
            .add_image_url("https://example.com/image2.jpg".to_string())
            .build();

        assert_eq!(variant.sku, "SKU123");
        assert_eq!(variant.abbreviated_color, Some("RED".to_string()));
        assert_eq!(variant.abbreviated_size, Some("M".to_string()));
        assert_eq!(variant.height, Some(10.0));
        assert_eq!(variant.width, Some(5.0));
        assert_eq!(variant.length, Some(2.0));
        assert_eq!(variant.weight, Some(0.5));
        assert_eq!(variant.weight_unit, Some("kg".to_string()));
        assert_eq!(variant.image_urls.len(), 2);
        
        if let Some(defining_attrs) = &variant.defining_attributes {
            assert_eq!(defining_attrs.get("size"), Some(&"Medium".to_string()));
        }
    }

    #[test]
    fn test_complete_product_with_variants() {
        // Test creating a complete product with variants using builders
        let packaging = PackagingBuilder::new()
            .dimensions(15.0, 10.0, 5.0)
            .weight(0.8, "kg".to_string())
            .build();

        let variant = ProductVariantBuilder::new("SKU456".to_string())
            .abbreviated_color("BLUE".to_string())
            .abbreviated_size("L".to_string())
            .dimensions(12.0, 8.0, 3.0)
            .weight(0.6, "kg".to_string())
            .packaging(packaging)
            .add_image_url("https://example.com/blue-large.jpg".to_string())
            .build();

        let reviews = ReviewsBuilder::new(4.0, 25, 4)
            .build();

        let categories = HierarchicalCategoriesBuilder::new()
            .lvl0("Clothing".to_string())
            .lvl1("Men's Clothing".to_string())
            .lvl2("Shirts".to_string())
            .build();

        let product = ProductBuilder::new("Complete Test Product".to_string(), "COMPLETE001".to_string())
            .brand("Premium Brand".to_string())
            .long_description("A complete product with all features".to_string())
            .add_variant(variant)
            .reviews(reviews)
            .hierarchical_categories(categories)
            .created_by("product_manager".to_string())
            .build();

        assert_eq!(product.name, "Complete Test Product");
        assert_eq!(product.variants.len(), 1);
        assert!(product.reviews.is_some());
        assert!(product.hierarchical_categories.is_some());
        
        if let Some(variant) = product.variants.first() {
            assert_eq!(variant.sku, "SKU456");
            assert!(variant.packaging.is_some());
        }
    }

    #[test]
    fn test_builder_pattern_usage_examples() {
        // Example 1: Simple product creation
        let simple_product = ProductBuilder::new("Basic T-Shirt".to_string(), "BASIC001".to_string())
            .brand("BasicWear".to_string())
            .display_on_site(true)
            .build();
        
        assert_eq!(simple_product.name, "Basic T-Shirt");
        assert_eq!(simple_product.product_ref, "BASIC001");
        assert!(simple_product.id.is_some());
        
        // Example 2: Product with multiple variants
        let variant1 = ProductVariantBuilder::new("SHIRT-S-RED".to_string())
            .abbreviated_color("RED".to_string())
            .abbreviated_size("S".to_string())
            .add_image_url("https://example.com/shirt-s-red.jpg".to_string())
            .build();
            
        let variant2 = ProductVariantBuilder::new("SHIRT-M-BLUE".to_string())
            .abbreviated_color("BLUE".to_string())
            .abbreviated_size("M".to_string())
            .add_image_url("https://example.com/shirt-m-blue.jpg".to_string())
            .build();
        
        let multi_variant_product = ProductBuilder::new("Colorful Shirt".to_string(), "COLOR001".to_string())
            .brand("FashionWear".to_string())
            .add_variant(variant1)
            .add_variant(variant2)
            .add_list_category("clothing".to_string())
            .add_list_category("shirts".to_string())
            .build();
            
        assert_eq!(multi_variant_product.variants.len(), 2);
        assert_eq!(multi_variant_product.list_categories.len(), 2);
        
        // Example 3: Using UUID generation for product ID
        let product_with_uuid = ProductBuilder::new("UUID Product".to_string(), "UUID001".to_string())
            .build();
            
        if let Some(id) = &product_with_uuid.id {
            // Validate UUID format (version 4 UUID is 36 characters with hyphens)
            assert_eq!(id.len(), 36);
            assert_eq!(id.chars().filter(|&c| c == '-').count(), 4);
        }
    }

    #[test]
    fn test_bayesian_average_rounding() {
        // Test that values are rounded to one decimal place
        let avg1 = BayesianAverage::new(4.56789);
        assert_eq!(avg1.as_f32(), 4.6);

        let avg2 = BayesianAverage::new(3.14159);
        assert_eq!(avg2.as_f32(), 3.1);

        let avg3 = BayesianAverage::new(2.95);
        assert_eq!(avg3.as_f32(), 3.0);
    }

    #[test]
    fn test_bayesian_average_from_trait() {
        let avg: BayesianAverage = 4.567.into();
        assert_eq!(avg.as_f32(), 4.6);
    }

    #[test]
    fn test_reviews_serialization() {
        let reviews = Reviews {
            bayesian_avg: BayesianAverage::new(4.56789),
            count: 100,
            rating: 5,
        };

        let json = serde_json::to_string(&reviews).expect("Failed to serialize");
        let deserialized: Reviews = serde_json::from_str(&json).expect("Failed to deserialize");

        // Check that the value is properly rounded
        assert_eq!(deserialized.bayesian_avg.as_f32(), 4.6);
        assert_eq!(deserialized.count, 100);
        assert_eq!(deserialized.rating, 5);
    }

    #[test]
    fn test_json_deserialization_with_precision() {
        let json = r#"{"bayesian_avg": 4.56789, "count": 100, "rating": 5}"#;
        let reviews: Reviews = serde_json::from_str(json).expect("Failed to deserialize");
        
        // Should be automatically rounded to one decimal place
        assert_eq!(reviews.bayesian_avg.as_f32(), 4.6);
    }
}
