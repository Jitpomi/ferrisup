//! Library template created with FerrisUp

pub mod fs;

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

/// Convert a string to snake_case
pub fn to_snake_case(s: &str) -> String {
    if s.is_empty() {
        return String::new();
    }
    
    let mut result = String::new();
    let mut last_was_underscore = false;
    let chars: Vec<char> = s.chars().collect();
    
    // Handle the first character
    if chars[0].is_alphanumeric() {
        result.push(chars[0].to_ascii_lowercase());
    } else {
        last_was_underscore = true;
    }
    
    // Process the rest of the characters
    for i in 1..chars.len() {
        let c = chars[i];
        
        if c.is_alphanumeric() {
            // If current char is uppercase and previous char was lowercase or a number,
            // add an underscore before it
            if c.is_uppercase() && 
               i > 0 && 
               (chars[i-1].is_lowercase() || chars[i-1].is_numeric()) && 
               !last_was_underscore {
                result.push('_');
            }
            result.push(c.to_ascii_lowercase());
            last_was_underscore = false;
        } else if !last_was_underscore {
            result.push('_');
            last_was_underscore = true;
        }
    }
    
    // Remove trailing underscore if any
    if result.ends_with('_') {
        result.pop();
    }
    
    result
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
    
    #[test]
    fn test_to_snake_case() {
        // PascalCase to snake_case conversion
        assert_eq!(to_snake_case("HelloWorld"), "hello_world");
        
        // camelCase to snake_case conversion
        assert_eq!(to_snake_case("helloWorld"), "hello_world");
        
        // Single word
        assert_eq!(to_snake_case("Hello"), "hello");
        
        // Already snake_case
        assert_eq!(to_snake_case("hello_world"), "hello_world");
        
        // Empty string
        assert_eq!(to_snake_case(""), "");
        
        // String with spaces
        assert_eq!(to_snake_case("Hello World"), "hello_world");
        
        // String with mixed punctuation
        assert_eq!(to_snake_case("Hello, World!"), "hello_world");
        
        // String with consecutive non-alphanumeric characters
        assert_eq!(to_snake_case("Hello--World"), "hello_world");
        
        // String with leading non-alphanumeric characters
        assert_eq!(to_snake_case("-Hello"), "hello");
        
        // String with trailing non-alphanumeric characters
        assert_eq!(to_snake_case("Hello-"), "hello");
        
        // Mixed case with numbers
        assert_eq!(to_snake_case("Hello123World"), "hello123_world");
        
        // All caps
        assert_eq!(to_snake_case("HELLO_WORLD"), "hello_world");
    }
}
