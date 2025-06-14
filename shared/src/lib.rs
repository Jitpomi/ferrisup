//! Library template created with FerrisUp

/// Convert a string to PascalCase
/// For example: "hello_world" -> "HelloWorld"
#[allow(dead_code)]
pub fn to_pascal_case(input: &str) -> String {
    input
        .split('_')
        .map(|word| {
            if word.is_empty() {
                String::new()
            } else {
                let mut chars = word.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                }
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_pascal_case() {
        // Basic snake_case to PascalCase conversion
        assert_eq!(to_pascal_case("hello_world"), "HelloWorld");
        
        // Single word
        assert_eq!(to_pascal_case("hello"), "Hello");
        
        // Multiple underscores
        assert_eq!(to_pascal_case("hello_beautiful_world"), "HelloBeautifulWorld");
        
        // Already capitalized
        assert_eq!(to_pascal_case("Hello_World"), "HelloWorld");
        
        // Empty string
        assert_eq!(to_pascal_case(""), "");
        
        // String with leading underscore
        assert_eq!(to_pascal_case("_hello"), "Hello");
        
        // String with trailing underscore
        assert_eq!(to_pascal_case("hello_"), "Hello");
        
        // String with consecutive underscores
        assert_eq!(to_pascal_case("hello__world"), "HelloWorld");
        
        // String with mixed case
        assert_eq!(to_pascal_case("hello_World"), "HelloWorld");
    }
}
