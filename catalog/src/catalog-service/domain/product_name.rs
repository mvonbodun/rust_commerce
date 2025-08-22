use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug, Clone)]
pub struct ProductName(String);

impl ProductName {
    /// Parse a string slice into a ProductName type
    pub fn parse(s: impl Into<String>) -> Result<ProductName, String> {
        let s = s.into();

        // Check if empty
        if s.trim().is_empty() {
            return Err("Product name cannot be empty".to_string());
        }

        // Check if whitespace only
        if s.chars().all(|c| c.is_whitespace()) {
            return Err("Product name cannot be only whitespace".to_string());
        }

        // Check length (using graphemes for proper Unicode handling)
        let graphemes = s.graphemes(true).count();
        if graphemes > 256 {
            return Err(format!(
                "Product name is too long. Maximum 256 characters, got {graphemes}"
            ));
        }

        // Check for forbidden characters that might cause issues
        let forbidden_characters = ['/', '\\', '\0', '\n', '\r', '\t'];
        if s.chars().any(|c| forbidden_characters.contains(&c)) {
            return Err("Product name contains forbidden characters".to_string());
        }

        Ok(Self(s))
    }
}

impl AsRef<str> for ProductName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for ProductName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_string_is_rejected() {
        let name = "";
        assert!(ProductName::parse(name).is_err());
    }

    #[test]
    fn whitespace_only_is_rejected() {
        let name = "   ";
        assert!(ProductName::parse(name).is_err());
    }

    #[test]
    fn name_with_256_graphemes_is_valid() {
        let name = "a".repeat(256);
        assert!(ProductName::parse(name).is_ok());
    }

    #[test]
    fn name_longer_than_256_graphemes_is_rejected() {
        let name = "a".repeat(257);
        assert!(ProductName::parse(name).is_err());
    }

    #[test]
    fn valid_name_is_parsed_successfully() {
        let name = "Test Product 123";
        let result = ProductName::parse(name);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().as_ref(), name);
    }

    #[test]
    fn name_with_forbidden_characters_is_rejected() {
        let names = vec![
            "Product/Name",
            "Product\\Name",
            "Product\0Name",
            "Product\nName",
            "Product\rName",
            "Product\tName",
        ];

        for name in names {
            assert!(
                ProductName::parse(name).is_err(),
                "Name '{}' should be rejected",
                name.replace('\0', "\\0")
            );
        }
    }
}
