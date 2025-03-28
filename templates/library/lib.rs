//! Library template created with FerrisUp

/// Returns a greeting message
pub fn hello() -> String {
    "Hello from FerrisUp library template!".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hello() {
        assert_eq!(hello(), "Hello from FerrisUp library template!");
    }
}
