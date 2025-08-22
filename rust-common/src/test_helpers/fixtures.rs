use uuid::Uuid;

/// Test data generators for creating valid and invalid test data
///
/// Generate random string of specified length
pub fn random_string(length: usize) -> String {
    use rand::Rng;
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    let mut rng = rand::thread_rng();

    (0..length)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

/// Generate a unique identifier
pub fn unique_id() -> String {
    Uuid::new_v4().to_string()
}

/// Generate a unique product reference
pub fn unique_product_ref() -> String {
    format!("PROD-{}", Uuid::new_v4().to_string()[0..8].to_uppercase())
}

/// Generate a unique SKU
pub fn unique_sku() -> String {
    format!("SKU-{}", Uuid::new_v4().to_string()[0..8].to_uppercase())
}

/// Generate a unique order reference
pub fn unique_order_ref() -> String {
    format!("ORD-{}", Uuid::new_v4().to_string()[0..8].to_uppercase())
}

/// Generate a valid email address
pub fn valid_email() -> String {
    format!("test.{}@example.com", unique_id().replace("-", ""))
}

/// Generate a valid URL slug
pub fn valid_slug() -> String {
    format!("test-product-{}", unique_id().replace("-", ""))
}

/// Invalid data generators for negative testing
pub mod invalid {
    /// Collection of SQL injection attempts
    pub fn sql_injection_strings() -> Vec<String> {
        vec![
            "'; DROP TABLE products; --".to_string(),
            "1 OR 1=1".to_string(),
            "admin'--".to_string(),
            "' OR '1'='1".to_string(),
            "\"; DROP TABLE products; --".to_string(),
            "1; DELETE FROM products WHERE 1=1".to_string(),
        ]
    }

    /// Collection of XSS attack attempts
    pub fn xss_strings() -> Vec<String> {
        vec![
            "<script>alert('XSS')</script>".to_string(),
            "javascript:alert('XSS')".to_string(),
            "<img src=x onerror=alert('XSS')>".to_string(),
            "<iframe src='javascript:alert(\"XSS\")'></iframe>".to_string(),
            "onclick=alert('XSS')".to_string(),
        ]
    }

    /// Collection of special characters that might break parsing
    pub fn special_characters() -> Vec<String> {
        vec![
            "!@#$%^&*()_+{}[]|\\:\";<>?,./".to_string(),
            "\n\r\t".to_string(),
            "\0".to_string(),
            "\\x00\\x01\\x02".to_string(),
            "🚀👾🎉".to_string(), // Unicode/emoji
            "مرحبا".to_string(),  // RTL text
            "τεστ".to_string(),   // Greek
        ]
    }

    /// Extremely long strings for boundary testing
    pub fn very_long_string(size_mb: usize) -> String {
        "A".repeat(size_mb * 1024 * 1024)
    }

    /// Empty and whitespace strings
    pub fn empty_strings() -> Vec<String> {
        vec![
            "".to_string(),
            " ".to_string(),
            "   ".to_string(),
            "\t".to_string(),
            "\n".to_string(),
        ]
    }

    /// Invalid numeric values
    pub fn invalid_numbers() -> Vec<String> {
        vec![
            "-1".to_string(),
            "0".to_string(),
            "-999999999".to_string(),
            "NaN".to_string(),
            "Infinity".to_string(),
            "1.5e308".to_string(), // Near max float
            "abc123".to_string(),
        ]
    }

    /// Invalid date strings
    pub fn invalid_dates() -> Vec<String> {
        vec![
            "not-a-date".to_string(),
            "2024-13-01".to_string(), // Invalid month
            "2024-02-30".to_string(), // Invalid day
            "1900-01-01".to_string(), // Too old
            "3000-01-01".to_string(), // Too far future
        ]
    }
}

/// Product test data builders
pub mod product {
    use super::*;
    use std::collections::HashMap;

    #[derive(Clone)]
    pub struct ProductBuilder {
        pub name: String,
        pub product_ref: String,
        pub slug: Option<String>,
        pub brand: Option<String>,
        pub long_description: Option<String>,
        pub product_type: Option<String>,
        pub display_on_site: bool,
        pub defining_attributes: HashMap<String, String>,
        pub descriptive_attributes: HashMap<String, String>,
    }

    impl Default for ProductBuilder {
        fn default() -> Self {
            Self {
                name: format!("Test Product {}", random_string(6)),
                product_ref: unique_product_ref(),
                slug: Some(valid_slug()),
                brand: Some("TestBrand".to_string()),
                long_description: Some("A test product description".to_string()),
                product_type: Some("simple".to_string()),
                display_on_site: true,
                defining_attributes: HashMap::new(),
                descriptive_attributes: HashMap::new(),
            }
        }
    }

    impl ProductBuilder {
        pub fn minimal() -> Self {
            Self {
                name: format!("Minimal Product {}", random_string(4)),
                product_ref: unique_product_ref(),
                slug: None,
                brand: None,
                long_description: None,
                product_type: None,
                display_on_site: false,
                defining_attributes: HashMap::new(),
                descriptive_attributes: HashMap::new(),
            }
        }

        pub fn with_variants() -> Self {
            let mut builder = Self::default();
            builder
                .defining_attributes
                .insert("color".to_string(), "red".to_string());
            builder
                .defining_attributes
                .insert("size".to_string(), "medium".to_string());
            builder
        }

        pub fn invalid_missing_name() -> Self {
            Self {
                name: "".to_string(),
                ..Self::default()
            }
        }

        pub fn invalid_missing_ref() -> Self {
            Self {
                product_ref: "".to_string(),
                ..Self::default()
            }
        }
    }
}

/// Category test data builders
pub mod category {
    use super::*;

    #[derive(Clone)]
    pub struct CategoryBuilder {
        pub name: String,
        pub slug: String,
        pub short_description: String,
        pub parent_id: Option<String>,
        pub display_order: i32,
    }

    impl Default for CategoryBuilder {
        fn default() -> Self {
            Self {
                name: format!("Test Category {}", random_string(4)),
                slug: valid_slug(),
                short_description: "A test category".to_string(),
                parent_id: None,
                display_order: 0,
            }
        }
    }

    impl CategoryBuilder {
        pub fn child_of(parent_id: String) -> Self {
            Self {
                parent_id: Some(parent_id),
                ..Self::default()
            }
        }

        pub fn root() -> Self {
            Self {
                name: format!("Root Category {}", random_string(4)),
                parent_id: None,
                ..Self::default()
            }
        }
    }
}

/// Inventory test data builders
pub mod inventory {
    use super::*;

    pub struct InventoryBuilder {
        pub sku: String,
        pub quantity: i32,
        pub reserved_quantity: i32,
        pub min_stock_level: i32,
        pub location: String,
    }

    impl Default for InventoryBuilder {
        fn default() -> Self {
            Self {
                sku: unique_sku(),
                quantity: 100,
                reserved_quantity: 0,
                min_stock_level: 10,
                location: "warehouse-1".to_string(),
            }
        }
    }

    impl InventoryBuilder {
        pub fn out_of_stock() -> Self {
            Self {
                quantity: 0,
                ..Self::default()
            }
        }

        pub fn low_stock() -> Self {
            Self {
                quantity: 5,
                min_stock_level: 10,
                ..Self::default()
            }
        }

        pub fn invalid_negative_quantity() -> Self {
            Self {
                quantity: -10,
                ..Self::default()
            }
        }
    }
}

/// Order test data builders
pub mod order {
    use super::*;

    pub struct OrderBuilder {
        pub order_ref: String,
        pub customer_ref: String,
        pub status: String,
    }

    impl Default for OrderBuilder {
        fn default() -> Self {
            Self {
                order_ref: unique_order_ref(),
                customer_ref: format!("CUST-{}", random_string(6)),
                status: "pending".to_string(),
            }
        }
    }

    pub struct AddressBuilder {
        pub name: String,
        pub address_line1: String,
        pub city: String,
        pub postal_code: String,
        pub country: String,
    }

    impl Default for AddressBuilder {
        fn default() -> Self {
            Self {
                name: "John Doe".to_string(),
                address_line1: "123 Test Street".to_string(),
                city: "Test City".to_string(),
                postal_code: "12345".to_string(),
                country: "US".to_string(),
            }
        }
    }
}

/// Price/Offer test data builders
pub mod price {
    use super::*;
    use chrono::{DateTime, Duration, Utc};

    pub struct OfferBuilder {
        pub sku: String,
        pub start_date: DateTime<Utc>,
        pub end_date: DateTime<Utc>,
        pub min_quantity: i32,
        pub max_quantity: Option<i32>,
        pub price: f64,
        pub currency: String,
    }

    impl Default for OfferBuilder {
        fn default() -> Self {
            let now = Utc::now();
            Self {
                sku: unique_sku(),
                start_date: now,
                end_date: now + Duration::days(30),
                min_quantity: 1,
                max_quantity: None,
                price: 99.99,
                currency: "USD".to_string(),
            }
        }
    }

    impl OfferBuilder {
        pub fn expired() -> Self {
            let now = Utc::now();
            Self {
                start_date: now - Duration::days(60),
                end_date: now - Duration::days(30),
                ..Self::default()
            }
        }

        pub fn future() -> Self {
            let now = Utc::now();
            Self {
                start_date: now + Duration::days(30),
                end_date: now + Duration::days(60),
                ..Self::default()
            }
        }

        pub fn invalid_dates() -> Self {
            let now = Utc::now();
            Self {
                start_date: now,
                end_date: now - Duration::days(1), // End before start
                ..Self::default()
            }
        }
    }
}
