/// Utility functions for templates

/// Convert a string to PascalCase
/// For example: "hello_world" -> "HelloWorld"
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
