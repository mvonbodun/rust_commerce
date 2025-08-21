use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug, Clone)]
pub struct ProductRef(String);

impl ProductRef {
    /// Parse a string slice into a ProductRef type
    pub fn parse(s: impl Into<String>) -> Result<ProductRef, String> {
        let s = s.into();
        
        // Check if empty
        if s.trim().is_empty() {
            return Err("Product reference cannot be empty".to_string());
        }
        
        // Check if whitespace only
        if s.chars().all(|c| c.is_whitespace()) {
            return Err("Product reference cannot be only whitespace".to_string());
        }
        
        // Check length
        let graphemes = s.graphemes(true).count();
        if graphemes > 100 {
            return Err(format!(
                "Product reference is too long. Maximum 100 characters, got {}",
                graphemes
            ));
        }
        
        if graphemes < 3 {
            return Err(format!(
                "Product reference is too short. Minimum 3 characters, got {}",
                graphemes
            ));
        }
        
        // Product refs should be alphanumeric with hyphens and underscores only
        if !s.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
            return Err(
                "Product reference can only contain letters, numbers, hyphens and underscores"
                    .to_string(),
            );
        }
        
        // Should not start or end with special characters
        if s.starts_with('-') || s.starts_with('_') || s.ends_with('-') || s.ends_with('_') {
            return Err(
                "Product reference cannot start or end with hyphens or underscores".to_string(),
            );
        }
        
        Ok(Self(s))
    }
}

impl AsRef<str> for ProductRef {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for ProductRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_string_is_rejected() {
        let reference = "";
        assert!(ProductRef::parse(reference).is_err());
    }

    #[test]
    fn whitespace_only_is_rejected() {
        let reference = "   ";
        assert!(ProductRef::parse(reference).is_err());
    }

    #[test]
    fn too_short_reference_is_rejected() {
        let reference = "AB";
        assert!(ProductRef::parse(reference).is_err());
    }

    #[test]
    fn too_long_reference_is_rejected() {
        let reference = "A".repeat(101);
        assert!(ProductRef::parse(reference).is_err());
    }

    #[test]
    fn valid_reference_is_parsed_successfully() {
        let references = vec![
            "PROD-12345",
            "SKU_ABC_123",
            "REF123",
            "product-ref-001",
        ];
        
        for reference in references {
            let result = ProductRef::parse(reference);
            assert!(result.is_ok(), "Reference '{}' should be valid", reference);
            assert_eq!(result.unwrap().as_ref(), reference);
        }
    }

    #[test]
    fn reference_with_invalid_characters_is_rejected() {
        let references = vec![
            "PROD 123",      // space
            "PROD@123",      // @
            "PROD#123",      // #
            "PROD/123",      // /
            "PROD\\123",     // \
            "PROD.123",      // .
        ];
        
        for reference in references {
            assert!(
                ProductRef::parse(reference).is_err(),
                "Reference '{}' should be rejected",
                reference
            );
        }
    }

    #[test]
    fn reference_starting_or_ending_with_special_chars_is_rejected() {
        let references = vec![
            "-PROD123",
            "_PROD123",
            "PROD123-",
            "PROD123_",
        ];
        
        for reference in references {
            assert!(
                ProductRef::parse(reference).is_err(),
                "Reference '{}' should be rejected",
                reference
            );
        }
    }
}