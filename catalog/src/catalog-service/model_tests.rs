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
